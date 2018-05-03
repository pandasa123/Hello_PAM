#!/bin/bash -e

# Response handling
prompt_yn () { 
  read -r -p "$1: " response
  if [[ -z "$response" ]]; then
    response="$2"
  fi
  if [[ "$response" =~ ^([yY][eE][sS]|[yY])+$ ]]; then
    true
  elif [[ "$response" =~ ^([nN][oO]|[nN])+$ ]]; then
    false
  else
    $(prompt_yn "$1" "$2")
  fi
}

# Echo function 
echo_stage () {
  echo -e "\e[32m$*\e[m"
}

pre_check(){
  if [ ! -e build/hello_pam.so ] || \
    [ ! -e build/HelloAuth/HelloAuth.exe ] || \
    [ ! -e build/CredentialCreator/CredentialCreator.exe ]; then
      echo "No built binary is found. Buld first before installing"
      exit 1
  fi
}

# Pre-check if executables are found
pre_check

# FIXED whitespace issue
WINUSER=`/mnt/c/Windows/System32/cmd.exe /C "echo | set /p temp=%username%"` 
if [[ ! "${WINUSER}" == "${WINUSER% *}" ]] ; then
    read WINUSER _ <<< $WINUSER
    END='~1'
    WINUSER=$(echo $WINUSER$END | tr [a-z] [A-Z]) 
fi


set +x
echo_stage "[1/7] Handling Windows installation location"
set -x

# Creating Windows PATHs 
DEF_hello_pam_WINPATH="/mnt/c/Users/$WINUSER/hello_pam"
echo "Input a Windows install location for Windows Hello authentication exe components."
echo -n "Default [${DEF_hello_pam_WINPATH}] :" 
read hello_pam_WINPATH
if [ -z "$hello_pam_WINPATH" ]; then
  hello_pam_WINPATH=$DEF_hello_pam_WINPATH
fi
if [ ! -e $hello_pam_WINPATH ]; then
  if prompt_yn "'$hello_pam_WINPATH' does not exist. Create it? [Y/n]" "y"; then
    set -x
    mkdir -p $hello_pam_WINPATH
  fi
fi

# Copying to WSL_HELLO_WINPATH
set +x
echo_stage "[2/7] Installing Windows components of WSL-Hello-sudo..."
set -x
cp -r build/{HelloAuth,CredentialCreator} "$hello_pam_WINPATH/"

# Copying hello_pam.so to linux + granting root permissions
set +x
echo_stage "[3/7] Installing PAM module to the Linux system..."
set -x
sudo cp build/hello_pam.so /lib/x86_64-linux-gnu/security/
sudo chown root:root /lib/x86_64-linux-gnu/security/hello_pam.so
sudo chmod 644 /lib/x86_64-linux-gnu/security/hello_pam.so

# Building config files with authentication path
set +x
echo_stage "[4/7] Creating the config files of WSL-Hello-sudo..."
set -x
sudo mkdir -p /etc/hello_pam/
set +x

if [ ! -e "/etc/hello_pam/config" ] || prompt_yn "'/etc/hello_pam/config' already exists. Overwrite it? [y/N]" "n" ; then
  set -x
  sudo touch /etc/hello_pam/config
  sudo echo "authenticator_path = \"$hello_pam_WINPATH/HelloAuth/HelloAuth.exe\"" | sudo tee /etc/hello_pam/config
else
  echo "skip creation of '/etc/hello_pam/config'"
fi

echo "Please authenticate yourself now to create a credential for '$USER' and '$WINUSER' pair."
KEY_ALREADY_EXIST_ERR=170
set -x
pushd $hello_pam_WINPATH
CredentialCreator/CredentialCreator.exe hello_pam_$USER|| test $? = $KEY_ALREADY_EXIST_ERR
sudo mkdir -p /etc/hello_pam/public_keys
popd
sudo cp "$hello_pam_WINPATH"/hello_pam_$USER.pem /etc/hello_pam/public_keys/

# Building an uninstall.sh with PATH
set +x
echo_stage "[5/7] Creating uninstall.sh..."
if [ ! -e "uninstall.sh" ] || prompt_yn "'uninstall.sh' already exists. Overwrite it? [Y/n]" "y" ; then
  cat > uninstall.sh << EOS
  echo -e "\e[31mNote: Please ensure that config files in /etc/pam.d/ are restored to as they were before WSL-Hello-sudo was installed\e[m"
  set -x
  sudo rm -rf /etc/hello_pam
  sudo rm /lib/x86_64-linux-gnu/security/hello_pam.so
  rm -rf ${hello_pam_WINPATH}
  grep -v "auth    sufficient    hello_pam.so" /etc/pam.d/sudo > temp; mv temp /etc/pam.d/sudo
EOS
  chmod +x uninstall.sh
else
  echo "skip creation of 'uninstall.sh'"
fi
set -x

set +x
echo_stage "[6/7] Configuring /etc/pam.d/sudo...Will need sudo!"
set -x

sudo sed -i 's/.*session    required   pam_env.so readenv=1 user_readenv=0.*/auth    sufficient    hello_pam.so\n&/' /etc/pam.d/sudo

set +x
echo_stage "[7/7] Finished installation and configuration! Reload shell and try `sudo whoami`"
set -x
echo "If you want to uninstall WSL-Hello-sudo, run uninstall.sh"
