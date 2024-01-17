.PHONY = test clean install

bindings: contracts/src/*.sol
	rm -rf bindings
	cd contracts && make build
	cd contracts && forge bind --bindings-path ../bindings --root . --crate-name bindings

install: bindings
	cargo install --path .

test: bindings
	cargo test -- --test-threads 1

clean:
	rm -rf bindings
	cd contracts && make clean

appimage:
	mkdir -p ~/Owshen-Production.AppDir/usr/bin
	mkdir -p ~/Owshen-Production.AppDir/usr/lib
	mkdir -p ~/Owshen-Production.AppDir/usr/share/applications
	mkdir -p ~/Owshen-Production.AppDir/usr/share/icons
	mkdir -p ~/Owshen-Production.AppDir/usr/share/owshen/client
	mkdir -p ~/Owshen-Production.AppDir/usr/share/networks
	mkdir -p ~/Owshen-Production.AppDir/usr/share/genesis

	cd target/release && cp ./owshen ~/Owshen-Production.AppDir/usr/bin

	cp -r client/build/* ~/Owshen-Production.AppDir/usr/share/owshen/client

	cd client/src/pics/icons && cp ./owshen.png ~/Owshen-Production.AppDir

	cp -r ./Sepolia.json ~/Owshen-Production.AppDir/usr/share/networks
	cp -r ./Sepolia-owshen-genesis.dat ~/Owshen-Production.AppDir/usr/share/genesis
	cp -r ./contracts/circuits/coin_withdraw_cpp/coin_withdraw ~/Owshen-Production.AppDir/usr/bin
	cp -r ./contracts/circuits/coin_withdraw_cpp/coin_withdraw.dat ~/Owshen-Production.AppDir/usr/bin
	cp -r ./contracts/circuits/coin_withdraw_0001.zkey ~/Owshen-Production.AppDir/usr/bin
	
	echo "[Desktop Entry]\nType=Application\nName=Owshen\nIcon=owshen\nExec=owshen\nCategories=Utility;" > ~/Owshen-Production.AppDir/owshen.desktop
	
	@echo '#!/bin/sh' > ~/Owshen-Production.AppDir/AppRun
	@echo 'SELF=$$(readlink -f "$$0")' >> ~/Owshen-Production.AppDir/AppRun
	@echo 'HERE=$${SELF%/*}' >> ~/Owshen-Production.AppDir/AppRun
	@echo '' >> ~/Owshen-Production.AppDir/AppRun
	@echo 'export LD_LIBRARY_PATH="$${HERE}/usr/lib/:$${LD_LIBRARY_PATH:+:$${LD_LIBRARY_PATH}}"' >> ~/Owshen-Production.AppDir/AppRun
	@echo 'export XDG_DATA_DIRS="$${HERE}/usr/share/$${XDG_DATA_DIRS:+:$${XDG_DATA_DIRS}}"' >> ~/Owshen-Production.AppDir/AppRun
	@echo '' >> ~/Owshen-Production.AppDir/AppRun
	@echo 'exec "$${HERE}/usr/bin/owshen" "$$@"' >> ~/Owshen-Production.AppDir/AppRun
	
	@chmod +x ~/Owshen-Production.AppDir/AppRun
	
	chmod +x ~/appimagetool-x86_64.AppImage
	
	mkdir -p ~/release

	~/appimagetool-x86_64.AppImage ~/Owshen-Production.AppDir

