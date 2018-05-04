# Hello PAM!

This is a Linux PAM module that uses Windows Hello for authentication whenever sudo is called. As with standard Windows Hello,
facial recognition, fingerprint authentication, PIN, and userspace password are all accepted for authentication

## Architecture

Hello PAM! is entirely built around Linux Pluggable Authentication Module and WSL Interoperatability. `sudo` calls the PAM module, PAM sends the user private RSA through `WSLENV`, Windows Hello creates a public RSA, and PAM compares it to the existing public RSA to authenticate. We span across WSL, WSLENV, and Windows-Environment to authenticate PAM via Windows Hello

[![Architecture](https://gitlab.eecs.umich.edu/pandasa/Hello_PAM/raw/master/Images/Architecture.png)](https://youtu.be/WM4jT1JHCOU)

## Installation

[![Installation Process](https://gitlab.eecs.umich.edu/pandasa/Hello_PAM/raw/master/Images/Installation.png)](https://youtu.be/WM4jT1JHCOU)

$ `git clone https://gitlab.eecs.umich.edu/pandasa/Hello_PAM.git`

$ `cd Hello_PAM`

$ `chmod +x install.sh`

$ `sudo ./install.sh`

## Building from source

$ `find "/mnt/c/Program Files (x86)/Microsoft Visual Studio" -name "MSBuild.exe" | grep -v "amd" | sed 's/ /\\ /g'` 

COPY and PASTE below

$ `export PATH=$PATH:[PASTE HERE]` 

$ `chmod +x packageinstall.sh`

$ `./packageinstall.sh`

$ `make`