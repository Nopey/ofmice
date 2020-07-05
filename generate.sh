#!/usr/bin/env bash
# run from just outside the open_fortress dir
root=`realpath .`
mkdir staging
staging=`realpath staging`
new=`realpath open_fortress`
old=`realpath open_fortress_bak`
cd "$new"

# important: lets the `find|while` loops modify our global variables
shopt -s lastpipe

function write_to_vpk(){
    echo "Packing '$vpk'.vpk"
    ~/vpk -P -M -c 500 k "$vpk" "$vpk.txt"
    #vpk -M -c 1000 k "$vpk" "$control"
}

function put_control_header(){
    # put a header in the soon-to-be-bak control file
    echo "// Open Fortress $bin vpk control file" | tee "$control" > "$control.new"
    echo "// Auto Generated by generate_control.sh" | tee -a "$control"  >> "$control.new"
}

function add_to_vpk(){
    # forbid uppercase until i figure out whats breaking vpk
    # [[ "$file" =~ [A-Z] ]] && return;

    md5=($(md5sum "$file"))
    # echo "$md5" > "$file.md5"
    destpath=$(echo "$file" | sed -e 's|^[^/]*/||')
    echo "\"$file\" { \"destpath\" \"$destpath\" \"md5\" \"$md5\" }" | tee -a "$control.new"  >> "$control"

    # the limit for a KeyValues file is 320,000 bytes.
    # so if we're over 300,000 bytes, we chunk it
    if [ -n "$(du -bt 300000 "$control")" ]; then
        next="$control.txt"
        echo "#base \"$next\"" >> "$vpk.txt"
        echo "#base \"$next.bak\"" >> "$vpk.txt.new"
        control="$next"
        put_control_header # creates the files, puts a comment
    fi
}

function add_dirs_to_vpk(){
    for dir in "${content[@]}"; do
        find "$dir" -type f -not -name '*.md5' -print0 |
        while IFS= read -r -d $'\0' file; do
            add_to_vpk
        done
    done
}

function start_vpk(){
    echo "Preparing control file for $vpk.vpk"
    control="$vpk.txt"
    # empty control
    : > "$control"
    # put a header in the soon-to-be-bak control file
    put_control_header
}

function end_vpk(){
    write_to_vpk
    control="$vpk.txt"
    while [ -e "$control.txt" ]; do
        mv "$control.txt.new" "$control.txt.bak"
        rm "$control.txt"
        ((i++))
    done
}

# of_textures
vpk=of_textures
start_vpk
find materials -type f -iname '*.vtf' -print0 |
while IFS= read -r -d $'\0' file; do
    add_to_vpk
done
end_vpk

# of_sound
vpk=of_sound
content=(sound)
start_vpk
add_dirs_to_vpk
end_vpk

# of_garnish
vpk=of_garnish
content=(classes expressions media particles resource)
start_vpk
add_dirs_to_vpk
end_vpk

# of_misc (server stuff, like tf2's misc)
vpk=of_misc
content=(models scenes scripts)
start_vpk
add_dirs_to_vpk
# server needs just the materials, not the textures.
find materials -type f -iname '*.vmt' -print0 |
while IFS= read -r -d $'\0' file; do
    add_to_vpk
done
end_vpk

echo "Done Building VPK's"
echo "Sleeping for 10 seconds to let the filesystem settle.."
sleep 10
echo "Building the tarballs.."

# Handles a loose file, thats in both the new and the old
function handle_new_file(){
    if [ -f "$old/$file" ]; then
        # file in both the new and old version
        if ! cmp --silent "$new/$file" "$old/$file"; then
            # Files differ
            mkdir -p `dirname $out/$file`
            /lib64/ld-linux-x86-64.so.2 ~/ddelta diff "$old/$file" "$new/$file" "$out/$file.dif"
        fi
    else
        # File Added
        mkdir -p `dirname $out/$file`
        cp "$new/$file" "$out/$file.new"
    fi
}

bins=(client server client_linux client_windows server_linux server_windows)

# TODO: figure out which of these loose files aren't needed. like whitelist.cfg
# NOTE: Deleting a file from these loose lists will not cause it to be deleted from user's machines.
client=(of_textures*.vpk of_sound*.vpk of_garnish*.vpk cfg custom/readme.txt download/readme.txt dxsupport_override.cfg lights.rad maplist.txt nospf.txt scripts steam.inf tfhallway.raw thirdpartycredits.txt)

server=(of_misc*.vpk README.md credits.txt gameinfo.txt gamemounting.txt maps whitelist.cfg)

#TODO: when GameUI.so works properly add it
client_linux=(bin/client.so bin/libdiscord-rpc.so bin/libfmod.so.11)
server_linux=(bin/server.so)
client_windows=(bin/client.dll bin/GameUI.dll bin/HLLib.dll bin/discord-rpc.dll bin/fmod.dll)
server_windows=(bin/server.dll)

for bin in "${bins[@]}"; do
    # Loose Files
    loose="${bin}[@]"
    files=("${!loose}")

    # prefix files
    files_with_prefix=()
    for file in "${files[@]}"; do
        files_with_prefix+=("open_fortress/${file}")
    done

    # Complete Tarball
    echo "Compressing ${bin} complete archive"
    tar cJf "${staging}/${bin}.tar.xz" -C "$root" "${files_with_prefix[@]}"

    # Diffing
    echo "Calculating changes for ${bin}"
    dif=$staging/$bin
    out=$dif/open_fortress
    for file in "${files[@]}"; do
        if [ -f "$new/$file" ]; then
            # File
            handle_new_file
        elif [ -d "$new/$file" ]; then
            # Directory
            # dirs can have deletions
            find "$dir" -type f -print0 |
            while IFS= read -r -d $'\0' file; do
                handle_new_file
            done
            find "$old/$dir" -type f -print0 |
            while IFS= read -r -d $'\0' file; do
                if [ ! -f "$new/$file" ]; then
                    # File deleted
                    mkdir -p `dirname "$out/$file"`
                    md5sum "$old/$file" > "$out/$file.del"
                fi
            done
        else
            echo "WARNING: $file ISN'T A FILE NOR A DIRECTORY"
        fi
    done
    echo "Compressing changes for ${bin}"
    tar cJf "${staging}/${bin}.tar.xz.dif" -C "$dif" open_fortress
    cd "$new"
    rm -rf "$dif"
done

echo Applying changes to Index
cd "$staging"
touch pending-deletion.txt
mkdir /var/www/html/of/mice/staging
mv *.tar.xz /var/www/html/of/mice/staging/ # just handle the clean installs here
~/ofpatchtool /var/www/html/of/mice/ 0 # manages the patches, generates a new index.json
mv /var/www/html/of/mice/staging/* /var/www/html/of/mice/
xargs -a pending-deletion.txt -d "\n" rm -rf
rm pending-deletion.txt

echo "Cleaning up for next run"
cd "$root"
rm -rf "$old"

# find "$old" -type f -name \*.md5

mkdir "$old"
# avoid .svn dir with /*
cp -rf "$new/*" "$old/"
rm "$old"/*.txt.bak