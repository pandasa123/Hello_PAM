
.PHONY: all clean install

all: build/hello_pam.so\
     build/HelloAuth/HelloAuth.exe\
     build/CredentialCreator/CredentialCreator.exe

build/hello_pam.so: build FORCE
	cargo build --release
	cp ./target/release/libhello_pam.so build/hello_pam.so

build/HelloAuth/HelloAuth.exe: build FORCE
	@if ! command -v MSBuild.exe > /dev/null; then \
	  echo "MSBuild.exe is not found in \$$PATH. Set the path to Visual Studio's MSBuild"; \
	  exit 1; \
	fi
	cd ./win_exe/HelloAuth;\
	MSBuild.exe "/t:Restore";\
	MSBuild.exe "/t:Build" "/p:Configuration=Release"
	mkdir -p build/HelloAuth
	cp ./win_exe/HelloAuth/HelloAuth/bin/Release/* build/HelloAuth/

build/CredentialCreator/CredentialCreator.exe: build FORCE
	@if ! command -v MSBuild.exe > /dev/null; then \
	  echo "MSBuild.exe is not found in \$$PATH. Set the path to Visual Studio's MSBuild"; \
	  exit 1; \
	fi
	cd ./win_exe/CredentialCreator;\
	MSBuild.exe "/t:Restore";\
	MSBuild.exe "/t:Build" "/p:Configuration=Release"
	mkdir -p build/CredentialCreator
	cp ./win_exe/CredentialCreator/CredentialCreator/bin/Release/* build/CredentialCreator/

FORCE:  ;
build:
	mkdir -p build

clean:
	cargo clean
	cd ./win_exe/CredentialCreator;\
	MSBuild.exe "/t:Clean" "/p:Configuration=Release"
	cd ./win_exe/HelloAuth;\
	MSBuild.exe "/t:Clean" "/p:Configuration=Release"
	rm -rf build
	rm -rf release
	rm release.tar.gz

install: all
	./install.sh

release: all
	mkdir -p release
	cp -R build release/
	cp install.sh release/
	tar cvzf release.tar.gz release
