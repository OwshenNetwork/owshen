import { createSlice, createSelector } from "@reduxjs/toolkit";

const containerSlice = createSlice({
  name: "container",
  initialState: {
    user: {
      address: null,
      name: null,
      image: null,
      nonce: null,
      // network main balances
      owshenBalance: {},
      availableBalances: null,
      chainDetails: null,
      sentCoins: [],
      receivedCoins: [],
      sentCoin: {},
      receivedCoinsLoading: null, //|| ,Number
    },
    owshen: {
      wallet: null,
      contract_address: null,
      contract_abi: null,
      dive_address: null,
      dive_abi: null,
      token_contracts: [],
      selected_token_contract: null,
      mode: null,
    },
    isTest: true,
    netWorkDetails: {
      name: "",
      chainId: 0,
      contractName: null,
    },
    isWalletConnected: false,
    isOwshenWalletExist: true,
  },
  reducers: {
    setUserDetails(state, { payload }) {
      state.user.address = payload.address;
      state.user.name = payload.name;
      state.user.image = payload.image;
      state.user.nonce = payload.nonce;
      state.user.balances = payload.balances;
      state.user.availableBalances = payload.availableBalances;
      state.user.chainDetails = payload.chainDetails;
    },
    setReceivedCoins(state, action) {
      switch (action.payload.type) {
        case "SET_RECEIVED_COINS":
          return {
            ...state,
            user: {
              ...state.user,
              receivedCoins: action.payload.payload,
            },
          };
        case "SET_CLOSEST_COIN":
          const amount = action.payload.payload;
          let result = {};
          if (state.user.receivedCoins.length > 0) {
            result = state.user.receivedCoins?.reduce((prev, curr) => {
              return Math.abs(curr.amount - Number(amount)) <
                Math.abs(prev.amount - Number(amount))
                ? curr
                : prev;
            });
          }

          return {
            ...state,
            user: {
              ...state.user,
              sentCoin: result,
            },
          };
        default:
          return state;
      }
    },

    setOwshen(state, action) {
      switch (action.payload.type) {
        case "SET_OWSHEN":
          return {
            ...state,
            owshen: {
              ...state.owshen,
              ...action.payload.payload,
            },
          };
        case "SET_SELECT_TOKEN_CONTRACT":
          return {
            ...state,
            owshen: {
              ...state.owshen,
              selected_token_contract: action.payload.payload,
            },
          };
        default:
          return state;
      }
    },
    setReceivedCoinsLoading(state, { payload }) {
      state.user.receivedCoinsLoading = payload;
    },
    setIsTest(state, { payload }) {
      state.isTest = payload;
    },
    setIsWalletConnected(state, { payload }) {
      state.isWalletConnected = payload;
    },
    setNetworkDetails(state, { payload }) {
      state.netWorkDetails.name = payload.name;
      state.netWorkDetails.chainId = payload.chainId;
      state.netWorkDetails.contractName = payload.contractName;
    },
    setIsOwshenWalletExist(state, { payload }) {
      state.isOwshenWalletExist = payload;
    },
  },
});

export const selectUserAddress = createSelector(
  (state) => state.container.user.address,
  (address) => address
);

export const selectUserOwshenBalance = createSelector(
  (state) => state.container.user.owshenBalance,
  (OB) => OB
);

export const selectReceivedCoins = createSelector(
  (state) => state.container.user.receivedCoins,
  (coin) => coin
);

export const selectOwshen = createSelector(
  (state) => state.container.owshen,
  (owshen) => owshen
);
export const selectReceivedCoinsLoading = createSelector(
  (state) => state.container.user.receivedCoinsLoading,
  (loading) => loading
);
export const selectIsTest = createSelector(
  (state) => state.container.isTest,
  (isTest) => isTest
);
export const selectIsWalletConnected = createSelector(
  (state) => state.container.isWalletConnected,
  (isWalletConnected) => isWalletConnected
);
export const selectSentCoin = createSelector(
  (state) => state.container.user.sentCoin,
  (coin) => coin
);
export const selectIsOwshenWalletExist = createSelector(
  (state) => state.container.isOwshenWalletExist,
  (isOwshenWalletExist) => isOwshenWalletExist
);
export const selectNetwork = createSelector(
  (state) => state.container.netWorkDetails,
  (netWorkDetails) => netWorkDetails
);

export const {
  setUserDetails,
  setReceivedCoins,
  setOwshen,
  setReceivedCoinsLoading,
  setIsTest,
  setNetworkDetails,
  setIsWalletConnected,
  setIsOwshenWalletExist,
} = containerSlice.actions;

export default containerSlice.reducer;

// necessary changes that have side effect

// import { createSlice, createSelector } from "@reduxjs/toolkit";
// import { toBigInt } from "ethers";

// const initialState = {
//   userAddress: null,
//   userName: null,
//   userImage: null,
//   userNonce: null,
//   owshenBalance: {},
//   availableBalances: null,
//   chainDetails: null,
//   sentCoins: [],
//   receivedCoins: [],
//   sentCoin: {},
//   receivedCoinsLoading: false,
//   owshenWallet: null,
//   contractAddress: null,
//   contractAbi: null,
//   diveAddress: null,
//   diveAbi: null,
//   tokenContracts: [],
//   selectedTokenContract: null,
//   isTest: false,
// };

// const containerSlice = createSlice({
//   name: "container",
//   initialState,
//   reducers: {
//     setUserDetails(state, { payload }) {
//       state.userAddress = payload.address;
//       state.userName = payload.name;
//       state.userImage = payload.image;
//       state.userNonce = payload.nonce;
//       state.availableBalances = payload.availableBalances;
//       state.chainDetails = payload.chainDetails;
//     },
//     setReceivedCoins(state, action) {
//       state.receivedCoins = action.payload;
//     },
//     setClosestCoin(state, action) {
//       const amount = action.payload;
//       let result = {};
//       if (state.receivedCoins.length > 0) {
//         result = state.receivedCoins.reduce((prev, curr) => {
//           return Math.abs(curr.amount - Number(amount)) <
//             Math.abs(prev.amount - Number(amount))
//             ? curr
//             : prev;
//         });
//       }
//       state.sentCoin = result;
//     },
//     setOwshenDetails(state, action) {
//       state.owshenWallet = action.payload.wallet;
//       state.contractAddress = action.payload.contract_address;
//       state.contractAbi = action.payload.contract_abi;
//       state.diveAddress = action.payload.dive_address;
//       state.diveAbi = action.payload.dive_abi;
//     },
//     setSelectedTokenContract(state, action) {
//       state.selectedTokenContract = action.payload;
//     },
//     setReceivedCoinsLoading(state, { payload }) {
//       state.receivedCoinsLoading = payload;
//     },
//     setIsTest(state, { payload }) {
//       state.isTest = payload;
//     },
//   },
// });

// export const {
//   setUserDetails,
//   setReceivedCoins,
//   setClosestCoin,
//   setOwshenDetails,
//   setSelectedTokenContract,
//   setReceivedCoinsLoading,
//   setIsTest
// } = containerSlice.actions;

// export default containerSlice.reducer;
