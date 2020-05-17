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
## To-Do
* installation::InstalledBin - Integrity checksum

* Integrate GTK with rest of logic in the Controller.
* Progress bar (module, with push/pop obj that gets passed around)
* Date (last successful update or up-to-date, build date)

* Index generation and patch trimming (bin)
* Write bash script that generates patches and tarballs from svn
    (basic patch logic already complete)
* Launcher self update detection
* Modify gameinfo.txt to hard-code TF2 Location. (hashsum it after)
* investigate russian translation

* Double check the licenses of the crates and steamworks.
* Double check builds are for i686 rather than x64
* (Cross?) Build Windows binaries
