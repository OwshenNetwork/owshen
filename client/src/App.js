import React from "react";
import "@fontsource/jetbrains-mono"; // Defaults to weight 400
import "@fontsource/jetbrains-mono/400.css"; // Specify weight
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import ReceivedCoinList from "./components/ReceivedCoins";
import Footer from "./components/Footer";
import Web3ModalComponent from "./components/walletConnection";
import Logo from "./pics/icons/logo.png";
import { Provider } from "react-redux";
import store from "./store/store";
import { WagmiProvider } from "wagmi";
import { config } from "./config";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import Drawer from "./components/Drawer";

const App = () => {
  const tokenInfo = [{ name: "Owshen", symbol: "DIVE" }];
  const queryClient = new QueryClient();
  //  const getCurrentChainId = async () => {
  //   let chainId = null;
  //   if (window.ethereum) {
  //      chainId = await window.ethereum.chainId;
  //   } else {
  //      console.log('Please install MetaMask!');
  //   }
  //   return console.log(chainId);
  //  };

  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <Provider store={store}>
          <div className="h-screen ">
            <ToastContainer theme="colored" />
            <div className="lg:mx-72 p-8 lg:mt-14 h-5/6 min-h-full lg:min-h-[726px] flex flex-col bg-white lg:rounded-lg lg:shadow-2xl">
              <div className="header justify-between flex w-full  items-center">
                <div className="hidden lg:flex w-3/6 justify-start ml-auto">
                  <Web3ModalComponent />
                </div>
                <div className="lg:hidden flex w-3/6 justify-start ml-auto">
                  <Drawer>
                    <Web3ModalComponent />
                  </Drawer>
                </div>

                <div className="flex w-4/6 items-center justify-end">
                  <img src={Logo} className="w-9 h-5 lg:w-[70px] lg:h-10" />
                  <h1 className="font-bold text-2xl lg:text-5xl pl-2 lg:pl-4">
                    Owshen
                  </h1>
                </div>
              </div>
              <Router basename="/">
                <Routes>
                  {tokenInfo.map((name) => (
                    <Route
                      key={name}
                      path={`/`}
                      element={<ReceivedCoinList />}
                    />
                  ))}
                </Routes>
              </Router>
              <Footer />
            </div>
          </div>
        </Provider>
      </QueryClientProvider>
    </WagmiProvider>
  );
};

export default App;
