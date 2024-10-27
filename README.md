# pprefox-rs

pprefox-rs is a client that works with pprefox, a Firefox browser extension. Together, they allow you to change your browser's theme *while it's open* over a simple HTTP API.

## What?

First, install [pprefox](https://github.com/duckfromdiscord/pprefox). Then, run pprefox-rs from the command line. It will install and let Firefox know that when the pprefox extension loads, the pprefox-rs executable needs to be run in "serve" mode. When run by Firefox, pprefox-rs will act as a web server that can be accessed by any program on your computer to change your browser theme. To get a list of your currently installed themes, make a GET request to `http://127.0.0.1:8080/get_themes`. You will get a JSON array of theme IDs and names. To set a theme, make a GET request to `http://127.0.0.1:8080/set_theme?id=theme_id` and replace `theme_id` with the ID of any theme in your list.

## How?

Let's start at the browser, and work our way outward to pprefox-rs. Unfortunately, there is no way to run a HTTP server inside your browser extension, so we can't just install an extension and communicate with it over HTTP. You must use a new protocol called [Native messaging](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging), which (under Windows) requires our program to create a registry key in Mozilla's section. It then requires you to point that registry key to an executable file, or script somewhere on the filesystem. When the extension starts, it looks for this entry, and runs that script. Then, once this script is run, all communication is done purely over stdio.

The fact this is our only option is really unfortunate because it is somewhat of a security concern: [as the Rust doc says](https://doc.rust-lang.org/std/env/fn.current_exe.html#security), if our executable gets replaced, Firefox executes a different executable from the one we want, and that could lead to privilege escalation. Therefore, you must make sure to keep your executable in the same spot and not writeable by any other software on your computer. There is not much else I can do about this besides use another means of communication with the extension. Without getting external servers involved, there is really only one other way to do this, and that is by having our program host a web server locally for the extension to connect to. This will lead to connection spam whenever our program is not running, which is not optimal It is also difficult to use async features while communicating over stdio. To solve this, I had to create a thread dedicated to reading from stdin, and another dedicated to writing to stdout, with the final one being the HTTP thread.

Some of the stdio handling code came from [guest271314/NativeMessagingHosts](https://github.com/guest271314/NativeMessagingHosts/blob/main/nm_rust.rs). Everything had to be adapted to async Rust, and split into separate threads.