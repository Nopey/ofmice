# OF Mice
The Open Fortress launcher.

## Building
To build, you will need to download the Steamworks SDK, and set
the `STEAM_SDK_LOCATION` environmental variable to the
installation directory of the SDK.

TODO: Other miscellanious dependencies
(use ldd to find out what libs it uses)

## Finished
* Basic GTK skeleton
* Pull SSDK path from steamworks and ensure TF2 is installed.
* The most basic level of signature checking
## TODO:
* Download and parse index
* Patch apply logic
* Open Fortress Install Dir detection
* Run Game
* Double check the licenses of the crates and steamworks.
* Integrate GTK with rest of logic in the Controller.
* Add another binary for generating updates
* Patch generation
* Seperating Windows and Linux binaries into seperate patches
* Logic for limiting patch number
* Signing
* Build for i686 rather than x64
* (Cross?) Build Windows binaries