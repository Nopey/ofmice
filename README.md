# OF Mice
The Open Fortress launcher.

## Building
there's a --feature steam_wrangler that requires the steam sdk to be installed and env var `STEAM_SDK_LOCATION` to be set to it.

TODO: Other miscellanious system dependencies
(use ldd to find out what libs it uses)

## Finished
* Basic GTK skeleton
* Pull SSDK path from steamworks and ensure TF2 is installed.
* Download index, clean install, patch apply
* run game
* Launch options
* Progress bar (module, with push/pop obj that gets passed around)
* make button look like update button when updates needed
* Download progress bar

## To-Do
### Backend things
* installation::InstalledBin - Integrity checksum
* Date (last successful update or up-to-date, build date)
* Index generation and patch trimming (bin)
* Write bash script that generates patches and tarballs from svn
    (basic patch logic already complete)
* Modify gameinfo.txt to hard-code TF2 Location. (hashsum it after)
* `ofserver` binary that is basically `ofmice` but for dedicated servers

### UI things
* disable launch button if it outlook is bad
* investigate russian translation
* file browser for config panel, see https://developer.gnome.org/gtk3/stable/GtkFileChooserDialog.html

### distribution things
* Launcher update detection (go into offline mode if user says they dont care)
* Double check the licenses of the crates and steamworks.
* Double check builds are for i686 rather than x64
* (Cross?) Build Windows binaries
* Launcher self update, replacement

## Grabbed from the code things
```
src/download.rs:    //TODO: Launcher self update detection
src/download.rs://TODO: Abstract the file download logic (for patches and full-bins) into its own function
src/download.rs://TODO: Check for the existance of .tar.xz files in the installation path with matching hashsums.
src/download.rs:            todo!() // test this, It looks good, but may not work.
src/download.rs:            //TODO: Download progress within the file for patches
src/progress.rs:        //TODO: include a (4/6 in the message by moving some stuff around)
src/main.rs:        INST.load().save_changes().expect("TODO: FIXME: THIS SHOULD DISPLAY AN ERR TO USER");
src/main.rs:        //TODO: change main button from UPDATE to PLAY
src/main.rs:                model.ed.display_error("TODO: actually handle DownloadErrors properly");
src/main.rs:                // TODO: credits: List crates used and stuff, especially the MIT APACHE and BSD licensed ones.\nMaybe have a series of credits boxes
```