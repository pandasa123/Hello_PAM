#!/bin/bash -e

sudo sed -i 's/.*session    required   pam_env.so readenv=1 user_readenv=0.*/auth    sufficient    hello_pam.so\n&/' test.txt