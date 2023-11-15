import React, { useEffect, useState } from "react";
import axios from "axios";

import Web3ModalComponent from "../walletConnection";
import { closeSend, openSend } from "../../utils/helper";

import "../../styles/main.css";
import { utils } from "web3";
import {
  selectUserAddress,
  setOwshen,
  setReceivedCoins,
} from "../../store/containerSlice";
import ReceivedCoinList from "../receivedCoins/index";
import { useDispatch, useSelector } from "react-redux";
import { ethers } from "ethers";
import { useApprove } from "../../hooks/useApprove";
import Dropdown from "../dropDown";

import Logo from "../../pics/icons/logo.svg";
import CopyIcon from "../../pics/icons/copy.svg";
import SendIcon from "../../pics/icons/send.svg";
import SwapIcon from "../../pics/icons/swap.svg";
import BackArrow from "../../pics/icons/arrow.svg";

const Main = () => {
  const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
  const address = useSelector(selectUserAddress);
  const dispath = useDispatch();

  const [OwshenWallet, setOwshenWallet] = useState({});
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  const [tokenOptions, setTokenOptions] = useState([]);
  const [tokenAmount, setTokenAmount] = useState("");
  const [tokenContract, setTokenContract] = useState("");
  const walletOptions = [
    { title: "Your Ethereum Account", value: "Your Ethereum Account" },
    { title: "Your Owshen Account", value: "Your Owshen Account" },
  ];
  const [walletName, setWalletName] = useState("");

  useEffect(() => {
    axios.get(`${coreEndpoint}/info`).then((result) => {
      setOwshenWallet({
        wallet: result.data.address,
        contract_address: result.data.owshen_contract,
        contract_abi: result.data.owshen_abi,
        dive_address: result.data.dive_contract,
        dive_abi: result.data.erc20_abi,
        token_contracts: result.data.token_contracts,
      });
      dispath(
        setOwshen({
          type: "SET_OWSHEN",
          payload: {
            wallet: result.data.address,
            contract_address: result.data.owshen_contract,
            contract_abi: result.data.owshen_abi,
            dive_address: result.data.dive_contract,
            dive_abi: result.data.erc20_abi,
            token_contracts: result.data.token_contracts,
          },
        })
      );
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
    if (OwshenWallet) {
      const newTokenOptions = OwshenWallet.token_contracts?.map((c, id) => {
        return { title: `test${id}`, value: c };
      });
      setTokenOptions(newTokenOptions);
    }
  }, [OwshenWallet.wallet, OwshenWallet.contract_address, coreEndpoint]);

  useEffect(() => {
    const coinsIntervalId = setInterval(() => {
      axios.get(`${coreEndpoint}/coins`).then((result) => {
        dispath(setReceivedCoins(result.data.coins));
      });
    }, 5000);
    return () => clearInterval(coinsIntervalId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  useEffect(() => {
    // This code will run whenever `tokenContract` changes
    if (tokenContract) {
      dispath(
        setOwshen({ type: "SET_SELECT_TOKEN_CONTRACT", payload: tokenContract })
      );
    }
  }, [tokenContract]); // Add `tokenContract` as a dependency

  const { approve, allowance } = useApprove(
    tokenContract,
    address,
    OwshenWallet.contract_address,
    OwshenWallet.dive_abi
  );
  const getStealth = async () => {
    if (!address) return console.log("Connect your wallet first");
    await axios
      .get(`${coreEndpoint}/stealth`, {
        params: { address: destOwshenWallet },
      })
      .then(async (result) => {
        let abi = OwshenWallet.contract_abi;
        let provider = new ethers.BrowserProvider(window.ethereum);
        let contract = new ethers.Contract(
          OwshenWallet.contract_address,
          abi,
          provider
        );

        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const ax = utils.toBigInt(result.data.address.x);
        const ay = utils.toBigInt(result.data.address.y);
        const ex = utils.toBigInt(result.data.ephemeral.x);
        const ey = utils.toBigInt(result.data.ephemeral.y);

        const pubKey = [ax, ay];
        const ephemeral = [ex, ey];
        const to_wei_token_amount = utils.toWei(Number(tokenAmount), "ether");
        try {
          if (Number(allowance) < Number(tokenAmount))
            await approve(to_wei_token_amount);
          const tx = await contract.deposit(
            pubKey,
            ephemeral,
            tokenContract,
            utils.toBigInt(to_wei_token_amount),
            address,
            OwshenWallet.contract_address
          );
          await tx.wait();
          axios.get(`${coreEndpoint}/coins`).then((result) => {
            setReceivedCoins(result.data.coins);
          });
        } catch (error) {
          console.log(error, "Error while getting approve or deposit flow");
        }
      });
  };

  function shortenAddress(address) {
    const firstPart = address.substring(0, 6);
    const lastPart = address.substring(address.length - 4);
    return `${firstPart}...${lastPart}`;
  }
  const copyWalletAddress = () => {
    navigator.clipboard.writeText(OwshenWallet.wallet);
  };
  return (
    <>
      <div className="header justify-between flex w-full  ">
        <Web3ModalComponent />
        <div className="flex w-1/2 justify-end">
          <img src={Logo} width="70px" />
          <h1 className="font-bold text-5xl pl-4">Owshen</h1>
        </div>
      </div>
      <div className="modal" id="send-modal">
        <div className=" border-2 text-center w-1/4 p-5 my-[10%] mx-auto bg-white rounded-xl">
          <div>
            <div style={{ cursor: "pointer" }} onClick={closeSend}>
              <img src={BackArrow} />
            </div>
            <h3 className="font-bold text-3xl">Send</h3>
          </div>
          <p>Privately send ERC-20 tokens to Owshen users!</p>
          <div className="px-3 flex justify-between items-center relative">
            <p className="absolute top-5 text-blue-500 left-5 text-xl">
              Destination{" "}
            </p>
            <input
              className="rounded py-7 px-2 bg-white my-4 border w-full border-blue-500 focus:border-blue-500 active:border-blue-500 "
              onChange={(e) => setDstOwshenWallet(e.target.value)}
              type="text"
            />
          </div>
          <div className="px-3 flex justify-between items-center">
            <label>
              <b>From: </b>
            </label>
            <Dropdown
              label="choose your wallet"
              options={walletOptions}
              select={setWalletName}
              style="py-5"
            />
          </div>
          <div className="px-3 flex justify-between items-center mt-3">
            <label>
              <b>token: </b>
            </label>
            <Dropdown
              label="choose your token"
              options={tokenOptions}
              select={setTokenContract}
              token={true}
              style="py-5"
            />
          </div>
          <div className="px-3 flex justify-between items-center relative">
            <button className="border rounded-3xl px-3 absolute bottom-0 left-4 border-blue-500 text-blue-500">
              <small> max</small>
            </button>
            <label>
              <b>Amount:</b>
            </label>

            <input
              className="rounded py-5 px-2 bg-white my-4 border w-60 text-center"
              placeholder="Enter amount"
              onChange={(e) => setTokenAmount(e.target.value)}
              type="text"
            />
          </div>
          <button
            onClick={async () => await getStealth()}
            className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1"
          >
            Send
          </button>
        </div>
      </div>
      <div style={{ textAlign: "center" }}>
        <div className="mt-10 ">
          {/* 🌊 Owshen Wallet 🌊 */}
          {OwshenWallet.wallet && (
            <button
              onClick={copyWalletAddress}
              className="mt-4 rounded-2xl px-3 py-2 w-52 mx-auto justify-between border border-gray-300 bg-[#BBDCFBCC] ease-in-out duration-300  flex"
            >
              {shortenAddress(OwshenWallet.wallet)}
              <img className="ml-2" src={CopyIcon} />
            </button>
          )}

          <div className="text-3xl font-bold mt-4">0.0 DIVE</div>
          <div className="text-lg mt-4">0.0 ETH</div>
          <div className="my-8 flex justify-around w-32 mx-auto">
            <button onClick={openSend}>
              <img src={SendIcon} />
            </button>
            <button>
              <img src={SwapIcon} />
            </button>
          </div>
        </div>
      </div>
      <ReceivedCoinList />
    </>
  );
};

export default Main;
