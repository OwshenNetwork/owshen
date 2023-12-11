import { createSlice, createSelector } from "@reduxjs/toolkit";
import { toBigInt } from "ethers";

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
      receivedCoinsLoading: false,
    },
    owshen: {
      wallet: null,
      contract_address: null,
      contract_abi: null,
      dive_address: null,
      dive_abi: null,
      token_contracts: [],
      selected_token_contract: null,
    },
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

export const selectSentCoin = createSelector(
  (state) => state.container.user.sentCoin,
  (coin) => coin
);

export const {
  setUserDetails,
  setReceivedCoins,
  setOwshen,
  setReceivedCoinsLoading,
} = containerSlice.actions;

export default containerSlice.reducer;
