# Flatpak Commands and Features Reference

**Version:** Flatpak 1.16.1  
**Default Architecture:** x86_64  
**Supported Architectures:** x86_64, i386  
**Default Installation:** /var/lib/flatpak

---

## Application and Runtime Management Commands

### Core Operations
- **`install`** - Install an application or runtime
- **`update`** - Update an installed application or runtime
- **`uninstall`** - Uninstall an installed application or runtime
- **`list`** - List installed apps and/or runtimes
- **`info`** - Show info for installed app or runtime
- **`search`** - Search for remote apps/runtimes

### Advanced Management
- **`mask`** - Mask out updates and automatic installation
- **`pin`** - Pin a runtime to prevent automatic removal
- **`make-current`** - Specify default version to run
- **`repair`** - Repair flatpak installation
- **`create-usb`** - Put applications or runtimes onto removable media

---

## Running Applications Commands

- **`run`** - Run an application
- **`override`** - Override permissions for an application
- **`enter`** - Enter the namespace of a running application
- **`ps`** - Enumerate running applications
- **`kill`** - Stop a running application

---

## File Access Management Commands

- **`documents`** - List exported files
- **`document-export`** - Grant an application access to a specific file
- **`document-unexport`** - Revoke access to a specific file
- **`document-info`** - Show information about a specific file

---

## Permission Management Commands

- **`permissions`** - List permissions
- **`permission-remove`** - Remove item from permission store
- **`permission-set`** - Set permissions
- **`permission-show`** - Show app permissions
- **`permission-reset`** - Reset app permissions

---

## Remote Repository Management Commands

- **`remotes`** - List all configured remotes
- **`remote-add`** - Add a new remote repository (by URL)
- **`remote-modify`** - Modify properties of a configured remote
- **`remote-delete`** - Delete a configured remote
- **`remote-ls`** - List contents of a configured remote
- **`remote-info`** - Show information about a remote app or runtime

---

## Build Commands

### Build Workflow
- **`build-init`** - Initialize a directory for building
- **`build`** - Run a build command inside the build dir
- **`build-finish`** - Finish a build dir for export
- **`build-export`** - Export a build dir to a repository
- **`build-bundle`** - Create a bundle file from a ref in a local repository
- **`build-import-bundle`** - Import a bundle file
- **`build-sign`** - Sign an application or runtime
- **`build-update-repo`** - Update the summary file in a repository
- **`build-commit-from`** - Create new commit based on existing ref
- **`repo`** - Show information about a repo

---

## System Commands

- **`config`** - Configure flatpak
- **`history`** - Show history

---

## Command Details

### Install Command
**Usage:** `flatpak install [OPTION…] [LOCATION/REMOTE] [REF…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to install for
- `--no-pull` - Don't pull, only install from local cache
- `--no-deploy` - Don't deploy, only download to local cache
- `--no-related` - Don't install related refs
- `--no-deps` - Don't verify/install runtime dependencies
- `--no-auto-pin` - Don't automatically pin explicit installs
- `--no-static-deltas` - Don't use static deltas
- `--runtime` - Look for runtime with the specified name
- `--app` - Look for app with the specified name
- `--include-sdk` - Additionally install the SDK used to build the given refs
- `--include-debug` - Additionally install the debug info for the given refs and their dependencies
- `--bundle` - Assume LOCATION is a .flatpak single-file bundle
- `--from` - Assume LOCATION is a .flatpakref application description
- `--gpg-file=FILE` - Check bundle signatures with GPG key from FILE
- `--subpath=PATH` - Only install this subpath
- `-y, --assumeyes` - Automatically answer yes for all questions
- `--reinstall` - Uninstall first if already installed
- `--noninteractive` - Produce minimal output and don't ask questions
- `--or-update` - Update install if already installed
- `--sideload-repo=PATH` - Use this local repo for sideloads

### Update Command
**Usage:** `flatpak update [OPTION…] [REF…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to update for
- `--commit=COMMIT` - Commit to deploy
- `--force-remove` - Remove old files even if running
- `--no-pull` - Don't pull, only update from local cache
- `--no-deploy` - Don't deploy, only download to local cache
- `--no-related` - Don't update related refs
- `--no-deps` - Don't verify/install runtime dependencies
- `--no-static-deltas` - Don't use static deltas
- `--runtime` - Look for runtime with the specified name
- `--app` - Look for app with the specified name
- `--appstream` - Update appstream for remote
- `--subpath=PATH` - Only update this subpath
- `-y, --assumeyes` - Automatically answer yes for all questions
- `--noninteractive` - Produce minimal output and don't ask questions
- `--sideload-repo=PATH` - Use this local repo for sideloads

### Uninstall Command
**Usage:** `flatpak uninstall [OPTION…] [REF…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to uninstall
- `--keep-ref` - Keep ref in local repository
- `--no-related` - Don't uninstall related refs
- `--force-remove` - Remove files even if running
- `--runtime` - Look for runtime with the specified name
- `--app` - Look for app with the specified name
- `--all` - Uninstall all
- `--unused` - Uninstall unused
- `--delete-data` - Delete app data
- `-y, --assumeyes` - Automatically answer yes for all questions
- `--noninteractive` - Produce minimal output and don't ask questions

### List Command
**Usage:** `flatpak list [OPTION…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `-d, --show-details` - Show extra information
- `--runtime` - List installed runtimes
- `--app` - List installed applications
- `--arch=ARCH` - Arch to show
- `-a, --all` - List all refs (including locale/debug)
- `--app-runtime=RUNTIME` - List all applications using RUNTIME
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `name` - Show the name
- `description` - Show the description
- `application` - Show the application ID
- `version` - Show the version
- `branch` - Show the branch
- `arch` - Show the architecture
- `runtime` - Show the used runtime
- `origin` - Show the origin remote
- `installation` - Show the installation
- `ref` - Show the ref
- `active` - Show the active commit
- `latest` - Show the latest commit
- `size` - Show the installed size
- `options` - Show options
- `all` - Show all columns

### Search Command
**Usage:** `flatpak search [OPTION…] TEXT`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to search for
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `name` - Show the name
- `description` - Show the description
- `application` - Show the application ID
- `version` - Show the version
- `branch` - Show the application branch
- `remotes` - Show the remotes
- `all` - Show all columns

### Run Command
**Usage:** `flatpak run [OPTION…] APP [ARGUMENT…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to use
- `--command=COMMAND` - Command to run
- `--cwd=DIR` - Directory to run the command in
- `--branch=BRANCH` - Branch to use
- `-d, --devel` - Use development runtime
- `--runtime=RUNTIME` - Runtime to use
- `--runtime-version=VERSION` - Runtime version to use
- `--commit` - Run specified commit
- `--runtime-commit` - Use specified runtime commit
- `--sandbox` - Run completely sandboxed
- `-p, --die-with-parent` - Kill processes when the parent process dies
- `--parent-pid=PID` - Use PID as parent pid for sharing namespaces
- `--parent-expose-pids` - Make processes visible in parent namespace
- `--parent-share-pids` - Share process ID namespace with parent
- `--app-path=PATH` - Use PATH instead of the app's /app
- `--usr-path=PATH` - Use PATH instead of the runtime's /usr

**Sandboxing Options:**
- `--share=SHARE` - Share with host (network, ipc, etc.)
- `--unshare=SHARE` - Unshare with host
- `--socket=SOCKET` - Expose socket to app
- `--nosocket=SOCKET` - Don't expose socket to app
- `--device=DEVICE` - Expose device to app
- `--nodevice=DEVICE` - Don't expose device to app
- `--allow=FEATURE` - Allow feature
- `--disallow=FEATURE` - Don't allow feature
- `--filesystem=FILESYSTEM[:ro]` - Expose filesystem to app (:ro for read-only)
- `--nofilesystem=FILESYSTEM` - Don't expose filesystem to app
- `--env=VAR=VALUE` - Set environment variable
- `--env-fd=FD` - Read environment variables in env -0 format from FD
- `--unset-env=VAR` - Remove variable from environment
- `--persist=FILENAME` - Persist home directory subpath

**D-Bus Options:**
- `--log-session-bus` - Log session bus calls
- `--log-system-bus` - Log system bus calls
- `--log-a11y-bus` - Log accessibility bus calls
- `--no-a11y-bus` - Don't proxy accessibility bus calls
- `--a11y-bus` - Proxy accessibility bus calls (default except when sandboxed)
- `--no-session-bus` - Don't proxy session bus calls
- `--session-bus` - Proxy session bus calls (default except when sandboxed)
- `--no-documents-portal` - Don't start portals
- `--file-forwarding` - Enable file forwarding
- `--own-name=DBUS_NAME` - Allow app to own name on the session bus
- `--talk-name=DBUS_NAME` - Allow app to talk to name on the session bus
- `--no-talk-name=DBUS_NAME` - Don't allow app to talk to name on the session bus
- `--system-own-name=DBUS_NAME` - Allow app to own name on the system bus
- `--system-talk-name=DBUS_NAME` - Allow app to talk to name on the system bus
- `--system-no-talk-name=DBUS_NAME` - Don't allow app to talk to name on the system bus
- `--a11y-own-name=DBUS_NAME` - Allow app to own name on the a11y bus

**USB Options:**
- `--usb=VENDOR_ID:PRODUCT_ID` - Add USB device to enumerables
- `--nousb=VENDOR_ID:PRODUCT_ID` - Add USB device to hidden list
- `--usb-list=LIST` - A list of USB devices that are enumerable
- `--usb-list-file=FILENAME` - File containing a list of USB devices to make enumerable

**Policy Options:**
- `--add-policy=SUBSYSTEM.KEY=VALUE` - Add generic policy option
- `--remove-policy=SUBSYSTEM.KEY=VALUE` - Remove generic policy option

### Override Command
**Usage:** `flatpak override [OPTION…] [APP]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--reset` - Remove existing overrides
- `--show` - Show existing overrides

**Override Options:** (Same as run command sandboxing options)
- All sandboxing, D-Bus, USB, and policy options from `run` command

### Info Command
**Usage:** `flatpak info [OPTION…] NAME [BRANCH]`

**Key Options:**
- `--arch=ARCH` - Arch to use
- `--user` - Show user installations
- `--system` - Show system-wide installations
- `--installation=NAME` - Show specific system-wide installations
- `-r, --show-ref` - Show ref
- `-c, --show-commit` - Show commit
- `-o, --show-origin` - Show origin
- `-s, --show-size` - Show size
- `-m, --show-metadata` - Show metadata
- `--show-runtime` - Show runtime
- `--show-sdk` - Show sdk
- `-M, --show-permissions` - Show permissions
- `--file-access=PATH` - Query file access
- `-e, --show-extensions` - Show extensions
- `-l, --show-location` - Show location

### Remote-Add Command
**Usage:** `flatpak remote-add [OPTION…] NAME LOCATION`

**Key Options:**
- `--no-gpg-verify` - Disable GPG verification
- `--no-enumerate` - Mark the remote as don't enumerate
- `--no-use-for-deps` - Mark the remote as don't use for deps
- `--prio=PRIORITY` - Set priority (default 1, higher is more prioritized)
- `--subset=SUBSET` - The named subset to use for this remote
- `--title=TITLE` - A nice name to use for this remote
- `--comment=COMMENT` - A one-line comment for this remote
- `--description=DESCRIPTION` - A full-paragraph description for this remote
- `--homepage=URL` - URL for a website for this remote
- `--icon=URL` - URL for an icon for this remote
- `--default-branch=BRANCH` - Default branch to use for this remote
- `--collection-id=COLLECTION-ID` - Collection ID
- `--gpg-import=FILE` - Import GPG key from FILE (- for stdin)
- `--filter=FILE` - Set path to local filter FILE
- `--disable` - Disable the remote
- `--authenticator-name=NAME` - Name of authenticator
- `--authenticator-option=KEY=VALUE` - Authenticator option
- `--authenticator-install` - Autoinstall authenticator
- `--no-authenticator-install` - Don't autoinstall authenticator
- `--no-follow_redirect` - Don't follow the redirect set in the summary file
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--if-not-exists` - Do nothing if the provided remote exists
- `--from` - LOCATION specifies a configuration file, not the repo location

### Remotes Command
**Usage:** `flatpak remotes [OPTION…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `-d, --show-details` - Show remote details
- `--show-disabled` - Show disabled remotes
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `name` - Show the name
- `title` - Show the title
- `url` - Show the URL
- `collection` - Show the collection ID
- `subset` - Show the subset
- `filter` - Show filter file
- `priority` - Show the priority
- `options` - Show options
- `comment` - Show comment
- `description` - Show description
- `homepage` - Show homepage
- `icon` - Show icon
- `all` - Show all columns

### Remote-Info Command
**Usage:** `flatpak remote-info [OPTION…] REMOTE REF`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--arch=ARCH` - Arch to install for
- `--commit=COMMIT` - Commit to show info for
- `--runtime` - Look for runtime with the specified name
- `--app` - Look for app with the specified name
- `--log` - Display log
- `-r, --show-ref` - Show ref
- `-c, --show-commit` - Show commit
- `-p, --show-parent` - Show parent
- `-m, --show-metadata` - Show metadata
- `--show-runtime` - Show runtime
- `--show-sdk` - Show sdk
- `--cached` - Use local caches even if they are stale
- `--sideloaded` - Only list refs available as sideloads

### History Command
**Usage:** `flatpak history [OPTION…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--since=TIME` - Only show changes after TIME
- `--until=TIME` - Only show changes before TIME
- `--reverse` - Show newest entries first
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `time` - Show when the change happened
- `change` - Show the kind of change
- `ref` - Show the ref
- `application` - Show the application/runtime ID
- `arch` - Show the architecture
- `branch` - Show the branch
- `installation` - Show the affected installation
- `remote` - Show the remote
- `commit` - Show the current commit
- `old-commit` - Show the previous commit
- `url` - Show the remote URL
- `user` - Show the user doing the change
- `tool` - Show the tool that was used
- `version` - Show the Flatpak version
- `all` - Show all columns

### PS Command
**Usage:** `flatpak ps [OPTION…]`

**Key Options:**
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `instance` - Show the instance ID
- `pid` - Show the PID of the wrapper process
- `child-pid` - Show the PID of the sandbox process
- `application` - Show the application ID
- `arch` - Show the architecture
- `branch` - Show the application branch
- `commit` - Show the application commit
- `runtime` - Show the runtime ID
- `runtime-branch` - Show the runtime branch
- `runtime-commit` - Show the runtime commit
- `active` - Show whether the app is active
- `background` - Show whether the app is background
- `all` - Show all columns

### Documents Command
**Usage:** `flatpak documents [OPTION…] [APPID]`

**Key Options:**
- `--columns=FIELD,…` - What information to show

**Available Columns:**
- `id` - Show the document ID
- `path` - Show the document path
- `origin` - Show the document path
- `application` - Show applications with permission
- `permissions` - Show permissions for applications
- `all` - Show all columns

### Config Command
**Usage:** `flatpak config [OPTION…] [KEY [VALUE]]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--list` - List configuration keys and values
- `--get` - Get configuration for KEY
- `--set` - Set configuration for KEY to VALUE
- `--unset` - Unset configuration for KEY

### Mask Command
**Usage:** `flatpak mask [OPTION…] [PATTERN…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--remove` - Remove matching masks

### Pin Command
**Usage:** `flatpak pin [OPTION…] [PATTERN…]`

**Key Options:**
- `-u, --user` - Work on the user installation
- `--system` - Work on the system-wide installation (default)
- `--installation=NAME` - Work on a non-default system-wide installation
- `--remove` - Remove matching pins

### Build-Init Command
**Usage:** `flatpak build-init [OPTION…] DIRECTORY APPNAME SDK RUNTIME [BRANCH]`

**Key Options:**
- `--arch=ARCH` - Arch to use
- `-v, --var=RUNTIME` - Initialize var from named runtime
- `--base=APP` - Initialize apps from named app
- `--base-version=VERSION` - Specify version for --base
- `--base-extension=EXTENSION` - Include this base extension
- `--extension-tag=EXTENSION_TAG` - Extension tag to use if building extension
- `-w, --writable-sdk` - Initialize /usr with a writable copy of the sdk
- `--type=TYPE` - Specify the build type (app, runtime, extension)
- `--tag=TAG` - Add a tag
- `--sdk-extension=EXTENSION` - Include this sdk extension in /usr
- `--extension=NAME=VARIABLE[=VALUE]` - Add extension point info
- `--sdk-dir=DIR` - Where to store sdk (defaults to 'usr')
- `--update` - Re-initialize the sdk/var

### Build Command
**Usage:** `flatpak build [OPTION…] DIRECTORY [COMMAND [ARGUMENT…]]`

**Key Options:**
- `-r, --runtime` - Use Platform runtime rather than Sdk
- `--readonly` - Make destination readonly
- `--bind-mount=DEST=SRC` - Add bind mount
- `--build-dir=DIR` - Start build in this directory
- `--sdk-dir=DIR` - Where to look for custom sdk dir (defaults to 'usr')
- `--metadata=FILE` - Use alternative file for the metadata
- `-p, --die-with-parent` - Kill processes when the parent process dies
- `--with-appdir` - Export application homedir directory to build

**Sandboxing Options:** (Same as run command)

### Build-Finish Command
**Usage:** `flatpak build-finish [OPTION…] DIRECTORY`

**Key Options:**
- `--command=COMMAND` - Command to set
- `--require-version=MAJOR.MINOR.MICRO` - Flatpak version to require
- `--no-exports` - Don't process exports
- `--extra-data` - Extra data info
- `--extension=NAME=VARIABLE[=VALUE]` - Add extension point info
- `--remove-extension=NAME` - Remove extension point info
- `--extension-priority=VALUE` - Set extension priority (only for extensions)
- `--sdk=SDK` - Change the sdk used for the app
- `--runtime=RUNTIME` - Change the runtime used for the app
- `--metadata=GROUP=KEY[=VALUE]` - Set generic metadata option
- `--no-inherit-permissions` - Don't inherit permissions from runtime

**Sandboxing Options:** (Same as run command)

### Build-Export Command
**Usage:** `flatpak build-export [OPTION…] LOCATION DIRECTORY [BRANCH]`

**Key Options:**
- `-s, --subject=SUBJECT` - One line subject
- `-b, --body=BODY` - Full description
- `--arch=ARCH` - Architecture to export for (must be host compatible)
- `-r, --runtime` - Commit runtime (/usr), not /app
- `--update-appstream` - Update the appstream branch
- `--no-update-summary` - Don't update the summary
- `--files=SUBDIR` - Use alternative directory for the files
- `--metadata=FILE` - Use alternative file for the metadata
- `--gpg-sign=KEY-ID` - GPG Key ID to sign the commit with
- `--exclude=PATTERN` - Files to exclude
- `--include=PATTERN` - Excluded files to include
- `--gpg-homedir=HOMEDIR` - GPG Homedir to use when looking for keyrings
- `--subset=SUBSET` - Add to a named subset
- `--end-of-life=REASON` - Mark build as end-of-life
- `--end-of-life-rebase=ID` - Mark build as end-of-life, to be replaced with the given ID
- `--token-type=VAL` - Set type of token needed to install this commit
- `--timestamp=TIMESTAMP` - Override the timestamp of the commit
- `--collection-id=COLLECTION-ID` - Collection ID
- `--disable-fsync` - Do not invoke fsync()
- `--disable-sandbox` - Do not sandbox icon validator
- `--no-summary-index` - Don't generate a summary index

### Build-Bundle Command
**Usage:** `flatpak build-bundle [OPTION…] LOCATION FILENAME NAME [BRANCH]`

**Key Options:**
- `--runtime` - Export runtime instead of app
- `--arch=ARCH` - Arch to bundle for
- `--repo-url=URL` - Url for repo
- `--runtime-repo=URL` - Url for runtime flatpakrepo file
- `--gpg-keys=FILE` - Add GPG key from FILE (- for stdin)
- `--gpg-sign=KEY-ID` - GPG Key ID to sign the OCI image with
- `--gpg-homedir=HOMEDIR` - GPG Homedir to use when looking for keyrings
- `--from-commit=COMMIT` - OSTree commit to create a delta bundle from
- `--oci` - Export oci image instead of flatpak bundle

---

## Global Options

### Information Options
- **`--version`** - Print version information and exit
- **`--default-arch`** - Print default arch and exit
- **`--supported-arches`** - Print supported arches and exit
- **`--gl-drivers`** - Print active gl drivers and exit
- **`--installations`** - Print paths for system installations and exit
- **`--print-updated-env`** - Print the updated environment needed to run flatpaks
- **`--print-system-only`** - Only include the system installation with --print-updated-env

### Verbosity Options
- **`-v, --verbose`** - Show debug information, -vv for more detail
- **`--ostree-verbose`** - Show OSTree debug information

### Help Options
- **`-h, --help`** - Show help options
- **`--help-all`** - Show all help options (for commands that support it)

---

## Installation Types

### User Installation
- **Location:** `~/.local/share/flatpak`
- **Access:** User-specific, no root required
- **Option:** `-u, --user`

### System Installation
- **Location:** `/var/lib/flatpak` (default)
- **Access:** System-wide, requires root
- **Option:** `--system` (default)

### Custom Installation
- **Option:** `--installation=NAME`
- Allows multiple system-wide installations

---

## Key Features

### Sandboxing
- Complete application isolation
- Configurable filesystem access
- Network isolation
- Device access control
- D-Bus filtering
- USB device management
- Environment variable control

### Permissions
- Dynamic permission management
- File access control via document portal
- Permission reset capabilities
- Runtime permission changes

### Runtimes and SDKs
- Shared runtime system
- SDK for building applications
- Extension support
- Base application support

### Remote Repositories
- Multiple remote support
- Priority-based repository selection
- GPG verification
- Collection ID support
- Authenticator support
- Filter files

### Build System
- Complete build workflow
- Bundle creation
- Repository export
- Signing support
- OCI image export
- Delta bundles

### Transaction Management
- History tracking
- Rollback capabilities
- Masking for update prevention
- Pinning for runtime protection

### File Management
- Document portal integration
- File export/import
- Persistent storage
- Home directory access control

### Performance
- Static deltas for efficient updates
- Local caching
- Sideloading support
- OSTree-based storage

---

## Common Share Types

When using `--share` and `--unshare`:
- **network** - Network access
- **ipc** - Inter-process communication
- **pid** - Process ID namespace
- **uts** - Hostname/domainname
- **user** - User namespace

---

## Common Socket Types

When using `--socket` and `--nosocket`:
- **x11** - X11 display server
- **wayland** - Wayland display server
- **pulseaudio** - PulseAudio sound server
- **session-bus** - D-Bus session bus
- **system-bus** - D-Bus system bus
- **a11y-bus** - Accessibility bus
- **pcsc** - Smart card access

---

## Common Device Types

When using `--device` and `--nodevice`:
- **dri** - Direct Rendering Infrastructure (GPU)
- **kvm** - Kernel-based Virtual Machine
- **shm** - Shared memory

---

## Common Feature Types

When using `--allow` and `--disallow`:
- **devel** - Development features
- **multiarch** - Multi-architecture support
- **bluetooth** - Bluetooth access
- **canbus** - CAN bus access

---

## Notes

- Flatpak uses OSTree for efficient storage and updates
- Applications run in isolated sandboxes by default
- Runtimes are shared between applications to save space
- User and system installations are separate
- All commands support extensive help via `--help` flag
- Column output supports ellipsization: `:s[tart]`, `:m[iddle]`, `:e[nd]`, `:f[ull]`
- Many commands support both user and system-wide operations
- Build commands provide a complete workflow for creating Flatpak applications
- Remote repositories can be configured with various authentication methods
- Permission system allows fine-grained control over application capabilities

