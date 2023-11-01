import { configureStore } from "@reduxjs/toolkit";
import containerSlice from "./containerSlice";

export default configureStore({
  reducer: {
    container: containerSlice,
  },
});
