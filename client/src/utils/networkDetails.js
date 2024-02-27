export const networkDetails = {
  Sepolia: {
    name: "Sepolia",
    chainId: 11155111,
    contractName: "Ethereum_Sepolia",
    metamaskChainId:'0xaa36a7'
  },
  Localhost: {
    name: "Localhost",
    chainId: 1337,
    contractName: "Localhost",
    metamaskChainId:'0x539'

  },
  Goerli: {
    name: "Goerli",
    chainId: 5,
    contractName: "Goerli",
    metamaskChainId:'0x5'

  },
  testnet: {
    name: "testnet",
    chainId: 5556,
    contractName: "Local-Testnet",
  },
};

export const getNetworkNameByChainId = (chainId) => {
    for (const key in networkDetails) {
       if (networkDetails[key].chainId === chainId) {
         return networkDetails[key].name;
       }
    }
    return null;
   };
   export const isChainIdExist = (chainId) => {
    for (const key in networkDetails) {
      if (networkDetails[key].chainId === chainId) {
        return true;
      }
    }
    return false;
  };