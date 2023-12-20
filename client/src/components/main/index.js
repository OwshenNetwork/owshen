import React, { useEffect, useState } from "react";
import axios from "axios";

import Web3ModalComponent from "../walletConnection";

import "../../styles/main.css";
import { utils } from "web3";
import {
  selectReceivedCoins,
  selectUserAddress,
  setOwshen,
  setReceivedCoins,
  setReceivedCoinsLoading,
  setIsTest,
} from "../../store/containerSlice";
import ReceivedCoinList from "../receivedCoins/index";
import { useDispatch, useSelector } from "react-redux";
import { ethers, toBigInt } from "ethers";
import { useApprove } from "../../hooks/useApprove";
import { toast } from "react-toastify";
import Dropdown from "../dropDown";
import Modal from "../Modal/Modal";
import { Tooltip } from "react-tooltip";
import { getERC20Balance } from "../../utils/helper";
import { currencies } from "../../utils/Currencies";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import InProgress from "../Modal/InProgress";
import Tokens from "../Tokens/Tokens";

import Logo from "../../pics/icons/logo.png";
import CopyIcon from "../../pics/icons/copy.png";
import SendIcon from "../../pics/icons/send.png";
import SwapIcon from "../../pics/icons/swap.png";
import BackArrow from "../../pics/icons/arrow.png";
import MetaMaskLogo from "../../pics/icons/metaMask.png";

const Main = () => {
  const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
  const address = useSelector(selectUserAddress);
  const receivedcoins = useSelector(selectReceivedCoins);
  const dispatch = useDispatch();

  const [OwshenWallet, setOwshenWallet] = useState({});
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  const [tokenOptions, setTokenOptions] = useState([]);
  const [tokenAmount, setTokenAmount] = useState(0);
  const [tokenContract, setTokenContract] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const [selectNetwork, setSelectNetWork] = useState("");
  const [isInprogress, setIsInprogress] = useState(false);
  const [isTest, setIsTestHere] = useState(false);

  const walletOptions = [
    {
      title: "Your Ethereum Account",
      value: "Your Ethereum Account",
      img: MetaMaskLogo,
    },
    { title: "Your Owshen Account", value: "Your Owshen Account", img: Logo },
  ];
  const netWorkOptions = [
    {
      title: "network1",
      value: "network1",
    },
    { title: "network2", value: "network2" },
  ];
  const [walletName, setWalletName] = useState("");
  const tokenInfo = [{ name: "owshen", symbol: "DIVE" }];

  useEffect(() => {
    axios.get(`${coreEndpoint}/info`).then(({ data }) => {
      setOwshenWallet({
        wallet: data.address,
        contract_address: data.owshen_contract,
        contract_abi: data.owshen_abi,
        dive_address: data.dive_contract,
        dive_abi: data.erc20_abi,
        token_contracts: data.token_contracts,
      });
      setIsTestHere(data.isTest);
      dispatch(
        setOwshen({
          type: "SET_OWSHEN",
          payload: {
            wallet: data.address,
            contract_address: data.owshen_contract,
            contract_abi: data.owshen_abi,
            dive_address: data.dive_contract,
            dive_abi: data.erc20_abi,
            token_contracts: data.token_contracts,
          },
        })
      );
      dispatch(setIsTest(data.isTest));
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
    if (OwshenWallet) {
      const newTokenOptions = OwshenWallet.token_contracts?.map(
        ({ symbol, token_address }, id) => {
          const img = currencies[symbol].img;

          return { title: symbol, value: token_address, img: img };
        }
      );
      setTokenOptions(newTokenOptions);
    }
  }, [OwshenWallet.wallet, OwshenWallet.contract_address, coreEndpoint]);

  useEffect(() => {
    dispatch(setReceivedCoinsLoading(true));

    const coinsIntervalId = setInterval(() => {
      axios.get(`${coreEndpoint}/coins`).then((result) => {
        dispatch(
          setReceivedCoins({
            type: "SET_RECEIVED_COINS",
            payload: result.data.coins,
          })
        );
        dispatch(setReceivedCoinsLoading(false));
      });
      // dispatch( setReceivedCoinsLoading(false))
    }, 5000);
    return () => clearInterval(coinsIntervalId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  useEffect(() => {
    // This code will run whenever `tokenContract` changes
    if (tokenContract) {
      dispatch(
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
  const checkIfTest = () => {
    if (isTest) {
      return setIsInprogress(true);
    }
  };
  const getStealth = async () => {
    if (!address) return toast.error("Connect your wallet first");
    if (!destOwshenWallet) return toast.error("Enter your Destination");
    if (!tokenContract) return toast.error("Select your token");
    if (!tokenAmount) return toast.error("enter amount of token");
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
            setIsOpen(false);
          });
        } catch (error) {
          toast.error("Error while getting approve or deposit flow");

          console.log(error, "Error while getting approve or deposit flow");
        }
      });
  };

  const findMatchingCoin = () => {
    for (let coin of receivedcoins) {
      if (
        coin.amount > tokenAmount &&
        String(coin.uint_token) === String(tokenContract)
      ) {
        return coin;
      }
    }

    return toast.error("No matching coin is found");
  };

  const send = async () => {
    if (!address) return toast.error("Connect your wallet first");
    if (!destOwshenWallet) return toast.error("Enter your Destination");
    if (!tokenContract) return toast.error("Select your token");
    if (!tokenAmount) return toast.error("enter amount of token");

    const selectedCoint = findMatchingCoin();

    const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
    const options = {
      gasLimit: 5000000,
    };
    const to_wei_token_amount = utils.toWei(Number(tokenAmount), "ether");

    axios

      .get(`${coreEndpoint}/send`, {
        params: {
          index: selectedCoint.index,
          address: OwshenWallet.wallet,
          new_amount: to_wei_token_amount,
          receiver_address: destOwshenWallet,
        },
      })
      .then(async (result) => {
        let abi = OwshenWallet.contract_abi;
        let commitment1 = result.data.sender_commitment;
        let commitment2 = result.data.receiver_commitment;
        let provider = new ethers.BrowserProvider(window.ethereum);

        let contract = new ethers.Contract(
          OwshenWallet.contract_address,
          abi,
          provider
        );

        let signer = await provider.getSigner();
        contract = contract.connect(signer);

        const proof = [
          result.data.proof.a,
          result.data.proof.b,
          result.data.proof.c,
        ];

        const rax = utils.toBigInt(result.data.receiver_ephemeral.x);
        const ray = utils.toBigInt(result.data.receiver_ephemeral.y);

        const receiver_ephemeral = [rax, ray];

        const sax = utils.toBigInt(result.data.sender_ephemeral.x);
        const say = utils.toBigInt(result.data.sender_ephemeral.y);

        const sender_ephemeral = [sax, say];

        try {
          const txResponse = await contract.send(
            result.data.nullifier,
            proof,
            receiver_ephemeral,
            sender_ephemeral,
            commitment1,
            commitment2,
            result.data.token,
            result.data.obfuscated_receiver_amount,
            result.data.obfuscated_sender_amount,
            true,
            options
          );
          console.log("Transaction response", txResponse);
          const txReceipt = await txResponse.wait();
          console.log("Transaction receipt", txReceipt);
        } catch (error) {
          toast.error("Error while getting withdraw flow");
          console.log(error, "Error while getting withdraw flow");
        }
      });
  };

  function shortenAddress(address) {
    const firstPart = address.substring(0, 6);
    const lastPart = address.substring(address?.length - 4);
    return `${firstPart}...${lastPart}`;
  }
  const copyWalletAddress = () => {
    navigator.clipboard.writeText(OwshenWallet.wallet);
    toast.success("your wallet address copied");
  };
  const canOpenModal = () => {
    if (isTest) {
      return setIsInprogress(true);
    }
    address ? setIsOpen(true) : toast.error("Connect your wallet first");
    // setWalletName("");
    setTokenAmount(0);
    // setTokenContract("");
    setDstOwshenWallet("");
  };
  const setMaxBalance = async () => {
    if (tokenContract && address && OwshenWallet.dive_abi) {
      const value = await getERC20Balance(
        tokenContract,
        address,
        OwshenWallet.dive_abi
      );
      return setTokenAmount(value);
    }
  };

  return (
    <>
      <InProgress isOpen={isInprogress} setIsOpen={setIsInprogress} />
      <div className="header justify-between flex w-full  ">
        <div className="flex w-3/6 justify-start ml-auto">
          <Web3ModalComponent />
        </div>
        <div className="flex w-4/6 justify-end">
          <img src={Logo} width="70px" />
          <h1 className="font-bold text-5xl pl-4">Owshen</h1>
        </div>
      </div>
      <Modal title="send" isOpen={isOpen} setIsOpen={setIsOpen}>
        <p>Privately send ERC-20 tokens to Owshen users!</p>
        <div className="px-3 flex justify-between items-center relative">
          <p className="absolute top-5 text-blue-500 left-5 text-xl">
            Destination
          </p>
          <input
            className="rounded py-7 px-2 bg-white my-4 border w-full border-blue-500 focus:border-blue-500 active:border-blue-500 "
            onChange={(e) => setDstOwshenWallet(e.target.value)}
            type="text"
            value={destOwshenWallet}
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
            style="py-5"
          />
        </div>
        <div className="px-3 flex justify-between items-center relative">
          <button
            onClick={setMaxBalance}
            className="border rounded-3xl px-3 absolute bottom-0 left-4 border-blue-500 text-blue-500"
          >
            <small> max</small>
          </button>
          <label>
            <b>Amount:</b>
          </label>

          <input
            className="rounded py-5 px-2 bg-white my-4 border w-60 text-center"
            placeholder="Enter amount"
            onChange={(e) => setTokenAmount(e.target.value)}
            type="number"
            value={tokenAmount}
          />
        </div>
        <button
          onClick={async () =>
            OwshenWallet.wallet === destOwshenWallet
              ? await getStealth()
              : await send()
          }
          className="border border-blue-400 bg-blue-200 text-blue-600 rounded-lg px-6 mt-3 font-bold py-1"
        >
          Send
        </button>
      </Modal>
      <div style={{ textAlign: "center" }}>
        <div className="mt-10 ">
          <Tooltip id="copy" place="top" content="copy wallet addres" />
          {/* 🌊 Owshen Wallet 🌊 */}
          {OwshenWallet.wallet && (
            <button
              data-tooltip-id="copy"
              onClick={copyWalletAddress}
              className="mt-4 rounded-2xl px-3 py-2 w-52 mx-auto justify-between border border-gray-300 bg-[#BBDCFBCC] ease-in-out duration-300  flex"
            >
              {shortenAddress(OwshenWallet.wallet)}
              <img className="ml-2" src={CopyIcon} />
            </button>
          )}

          <div className="text-3xl font-bold mt-4">0.0 DIVE</div>
          <div className="text-lg mt-4">$? USD</div>
          <div className="my-8 flex justify-around w-32 mx-auto">
            <Tooltip id="send" place="bottom" content="send" />
            <button data-tooltip-id="send" onClick={canOpenModal}>
              <img src={SendIcon} />
            </button>
            <Tooltip id="swap" place="bottom" content="swap" />
            <button data-tooltip-id="swap" onClick={canOpenModal}>
              <img src={SwapIcon} />
            </button>
          </div>
        </div>
      </div>
      <Router basename="/">
        <Routes>
          <Route path={"/"} element={<Tokens tokensInfo={tokenInfo} />} />

          {tokenInfo.map((name) => (
            <Route
              key={name}
              path={`/token/:name`}
              element={<ReceivedCoinList />}
            />
          ))}
        </Routes>
      </Router>
    </>
  );
};

export default Main;
