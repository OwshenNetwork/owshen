
# Targets
.PHONY: deploy verify upgrade

deploy:

	@echo "Deploying Owshen contract..."
	PRIVATE_KEY=$(PRIVATE_KEY) forge script script/Owshen.s.sol:DeployOwshen --broadcast --fork-url $(PROVIDER) | tee deploy_output.txt
	cat deploy_output.txt | grep "Transactions saved to:" | xargs -I @ python -c "print('@'.split(' ')[-1])" | xargs -I @ python -c "import json; d=json.load(open('@'));dt={'implementation_address': d['returns']['0']['value'], 'proxy_address': d['returns']['1']['value']}; dt.update({'abi': json.load(open('out/Owshen.sol/Owshen.json'))['abi']}); json.dump(dt, open('../config.json', 'w'), indent=4);"

verify:
	@echo "verifying logic contract ..."
	forge verify-contract --chain-id $(CHAIN_ID) --etherscan-api-key $(ETHERSCAN_API) $(CONTRACT_ADDRESS) Owshen 


upgrade:
	@echo "upgrading owshen ... "
	PRIVATE_KEY=$(PRIVATE_KEY) PROXY_ADDRESS=$(PROXY_ADDRESS) forge script script/UpgradeOwshen.s.sol:UpgradeOwshen --broadcast --fork-url $(PROVIDER) | tee upgrade_output.txt


clean:
	@echo "Cleaning build artifacts..."
	forge clean

build:
	@echo "Compiling contracts..."
	forge build



PRIVATE_KEY := 
PROVIDER := https://ethereum-sepolia-rpc.publicnode.com
ETHERSCAN_API := 
PROXY_ADDRESS := 
CONTRACT_ADDRESS := 
CHAIN_ID := 11155111
