# Hello PAM!

This is a Linux PAM module that uses Windows Hello for authentication whenever sudo is called. As with standard Windows Hello,
facial recognition, fingerprint authentication, PIN, and userspace password are all accepted for authentication

## Installation and Configuration

### Installation

$ `git clone https://gitlab.eecs.umich.edu/pandasa/Hello_PAM.git`
$ `cd Hello_PAM`
$ `chmod +x install.sh`
$ `./install.sh`

## Build

This following step depends on the current Visual Studio version but should give you an idea of where to look.
We're adding MSBuild.exe to WSL's path 
$ `export PATH=$PATH:/mnt/c/Program\ Files\ \(x86\)/Microsoft\ Visual\ Studio/2017/Community/MSBuild/15.0/Bin/` 
$ `chmod +x packageinstall.sh`
$ `./packageinstall.sh`
$ `make`