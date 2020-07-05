#!/usr/bin/env bash
# run from just outside the open_fortress dir
root=`realpath .`
mkdir staging
staging=`realpath staging`
new=`realpath open_fortress`
old=`realpath open_fortress_bak`
cd "$new"
DDELTA=ddelta
OFPATCHTOOL="$root/ofpatchtool"
WWW=/var/www/html/of/mice

# important: lets the `find|while` loops modify our global variables
shopt -s lastpipe

# NOTE: no longer vpk'ing, as it seems SSDK2013 on linux has just completely broken vpk's.
# Maybe someday. On that day, simply reference commit 81f5ceb1fb49d0b4d7ab272ec6ad74b8465bd063.

# Handles a loose file, thats in both the new and the old
function handle_new_file(){
    if [ -f "$old/$file" ]; then
        # file in both the new and old version
        if ! cmp --silent "$new/$file" "$old/$file"; then
            # Files differ
            mkdir -p `dirname $out/$file`
            $DDELTA diff "$old/$file" "$new/$file" "$out/$file.dif"
        fi
    else
        # File Added
        mkdir -p `dirname $out/$file`
        cp "$new/$file" "$out/$file.new"
    fi
}

bins=(client server client_linux client_windows server_linux server_windows)

# TODO: figure out which of these loose files aren't needed. like whitelist.cfg
# NOTE: Deleting a file from these loose lists will not cause it to be deleted from user's machines. (modify this script with caution)
# NOTE: The trailing slashes aren't necessary, but they do make it clear what is a file.
client=(sound/ classes/ expressions/ media/ particles/ resource cfg custom/readme.txt download/readme.txt dxsupport_override.cfg lights.rad maplist.txt nospf.txt scripts steam.inf tfhallway.raw thirdpartycredits.txt)

server=(models/ scenes/ scripts/ README.md credits.txt gameinfo.txt gamemounting.txt maps whitelist.cfg)

# I've decided to put the server.so in both client and server buckets,
# in case we ever start building server binaries that don't require symlinking _srv in ssdk2013.
client_linux=(bin/client.so bin/libdiscord-rpc.so bin/libfmod.so.11 bin/GameUI.so bin/game_shader_dx9.so bin/server.so)
server_linux=(bin/server.so)
client_windows=(bin/client.dll bin/GameUI.dll bin/HLLib.dll bin/discord-rpc.dll bin/fmod.dll bin/game_shader_dx9.dll bin/server.dll)
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

    # Complete Tarball (no diffing)
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
touch pending-deletion.txt # generated by ofpatchtool
mkdir $WWW/.wwwstaging
mv *.tar.xz $WWW/.wwwstaging/ # just handle the clean installs here
$OFPATCHTOOL $WWW/ 0 # manages the patches, generates a new index.json
mv $WWW/.wwwstaging/* $WWW/
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
