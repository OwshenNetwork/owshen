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
      balances: null,
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
    },
    setOwshen(state, { payload }) {
      state.owshen = {
        wallet: payload.wallet,
        contract_address: payload.contract_address,
        contract_abi: payload.contract_abi,
        dive_address: payload.dive_address,
        dive_abi: payload.dive_abi,
      };
    },
  },
});

export const selectUserAddress = createSelector(
  (state) => state.container.user.address,
  (address) => address
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
