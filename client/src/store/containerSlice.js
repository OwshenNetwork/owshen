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
    setReceivedCoins(state, { payload }) {
      state.user.receivedCoins = payload;

      const seenIndexes = new Set();
      let totalAmount = 0;

      payload?.forEach((coin) => {
        const amount = toBigInt(coin.amount).toString();
        if (!seenIndexes.has(coin.index)) {
          seenIndexes.add(coin.index);
          totalAmount += Number(amount);
          console.log("total amount", totalAmount);
          state.user.owshenBalance.ETH += totalAmount;
        }
      });
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

export const { setUserDetails, setReceivedCoins, setOwshen } =
  containerSlice.actions;

export default containerSlice.reducer;
