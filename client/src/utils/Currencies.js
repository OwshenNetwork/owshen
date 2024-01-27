import WETHIcon from "../pics/tokens/WETH.png";
// import DAIIcon from "../assets/img/common/DAI-icon.png";
// import USDTIcon from "../assets/img/common/USDT-icon.png";
import USDCIcon from "../pics/tokens/USDC.png";
import ETHIcon from "../pics/tokens/ETH.png";
import DIVEIcon from "../pics/tokens/Dive.png";
export const currencyIcons = {
  WETH: WETHIcon,
  // DAI: DAIIcon,
  // USDT: USDTIcon,
  USDC: USDCIcon,
};
export const currencies = {
  WETH: {
    name: "Wrapped Ether",
    decimals: 18,
    chain: {
      zksyncv1: {
        contract: "",
        L2Contract: 61,
      },
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "", // "0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6",
      },
      arbitrum: {
        contract: "",
      },
      arbitrum_goerli: {
        contract: "",
      },
      goerli: {
        contract: "0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6",
      },
      //// mainnet 👇 ////
      homestead: {
        contract: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
      },
    },
    img: WETHIcon,
  },
  ETH: {
    name: "Wrapped Ether",
    decimals: 18,
    chain: {
      zksyncv1: {
        contract: "",
        L2Contract: 61,
      },
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "", // "0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6",
      },
      arbitrum: {
        contract: "",
      },
      arbitrum_goerli: {
        contract: "",
      },
      goerli: {
        contract: "0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6",
      },
      //// mainnet 👇 ////
      homestead: {
        contract: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
      },
    },
    img: ETHIcon,
  },
  USDC: {
    name: "USDC",
    decimals: 6,
    chain: {
      zksyncv1: {
        contract: "",
        L2Contract: 2,
      },
      zksyncv1_goerli: {
        contract: "",
        L2Contract: 3,
      },
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "",
      },
      arbitrum: {
        contract: "",
      },
      goerli: {
        contract: "0x0aa78575e17ac357294bb7b5a9ea512ba07669e2",
      },
      //// mainnet 👇 ////
      homestead: {
        contract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      },
    },
    img: USDCIcon,
  },
  USDT: {
    name: "USDT",
    decimals: 6,
    chain: {
      zksyncv1: {
        contract: "",
        L2Contract: 4,
      },
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "",
      },
      arbitrum: {
        contract: "",
      },
      goerli: {
        contract: "0xb0f7554a44cc178e935ea10c79e7c042d1840044",
      },
      homestead: {
        contract: "0xdac17f958d2ee523a2206206994597c13d831ec7",
      },
    },
    img: WETHIcon,
  },
  DAI: {
    name: "DAI",
    decimals: 18,
    chain: {
      zksyncv1: {
        contract: "",
        L2Contract: 1,
      },
      zksyncv1_goerli: {
        contract: "",
        L2Contract: 4,
      },
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "",
      },
      arbitrum_goerli: {
        contract: "",
      },
      goerli: {
        contract: "0x9d233a907e065855d2a9c7d4b552ea27fb2e5a36",
      },
      //// mainnet 👇 ////
      homestead: {
        contract: "0x6b175474e89094c44da98b954eedeac495271d0f",
      },
    },
    img: WETHIcon,
  },

  DIVE: {
    name: "DIVE",
    decimals: 18,
    chain: {
      ethereum: {
        contract: "",
      },
      ethereum_goerli: {
        contract: "",
      },
      arbitrum_goerli: {
        contract: "",
      },
      goerli: {
        contract: "",
      },
      //// mainnet 👇 ////
      homestead: {
        contract: "",
      },
      sepolia: {
        contract: "0x4bf749ec68270027c5910220ceab30cc284c7ba2",
      },
    },
    img: DIVEIcon,
  },
};

export function getNetworkCurrencies(network) {
  const entries = Object.entries(currencies)
    .filter(([_, currency]) => currency.chain[network])
    .map(([ticker, currency]) => {
      const { chain, ...curr } = currency;
      return [
        ticker,
        {
          ...curr,
          info: chain[network],
        },
      ];
    });
  return Object.fromEntries(entries);
}
export function getNetworkCurrency(network, ticker) {
  if (!currencies[ticker]) {
    return null;
  }
  const { chain, decimals, ...curr } = currencies[ticker];
  return { ...curr, info: { ...chain?.[network], decimals } };
}

export function getCurrencyKeys() {
  return Object.keys(currencies);
}

export function getLogoByContractAddress(contractAddress) {
  for (const currency of Object.values(currencies)) {
    for (const chainInfo of Object.values(currency.chain)) {
      if (chainInfo.contract === contractAddress) {
        return currency.img;
      }
    }
  }

  return DIVEIcon;
}

export function getNameByContractAddress(contractAddress) {
  for (const [_, currency] of Object.entries(currencies)) {
    for (const chainInfo of Object.values(currency.chain)) {
      if (chainInfo.contract === contractAddress) {
        return currency.name;
      }
    }
  }
  return "DIVE";
}

export function getDecimalByContractAddress(contractAddress) {
  for (const [_, currency] of Object.entries(currencies)) {
    for (const chainInfo of Object.values(currency.chain)) {
      if (chainInfo.contract === contractAddress) {
        return currency.decimals;
      }
    }
  }
  return 18;
}
