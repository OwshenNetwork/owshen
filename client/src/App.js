import React, { useEffect } from "react";
import axios from "axios";
import "@fontsource/jetbrains-mono"; // Defaults to weight 400
import "@fontsource/jetbrains-mono/400.css"; // Specify weight
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Tokens from "./components/Tokens/Tokens";
import ReceivedCoinList from "./components/ReceivedCoins";
import Footer from "./components/Footer";
import Web3ModalComponent from "./components/walletConnection";
import Logo from "./pics/icons/logo.png";

const App = () => {
  const tokenInfo = [{ name: "Owshen", symbol: "DIVE" }];

  return (
    <div className="h-screen ">
      <ToastContainer theme="colored" />
      <div className="lg:mx-72 p-8 mt-14 h-5/6 min-h-[726px] flex flex-col bg-white rounded-lg shadow-2xl">
        <div className="header justify-between flex w-full  ">
          <div className="flex w-3/6 justify-start ml-auto">
            <Web3ModalComponent />
          </div>
          <div className="flex w-4/6 justify-end">
            <img src={Logo} width="70px" />
            <h1 className="font-bold text-5xl pl-4">Owshen</h1>
          </div>
        </div>
        <Router basename="/">
          <Routes>
            {tokenInfo.map((name) => (
              <Route key={name} path={`/`} element={<ReceivedCoinList />} />
            ))}
          </Routes>
        </Router>
        <Footer />
      </div>
    </div>
  );
};

export default App;
