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

const Main = () => {
  const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT;
  const address = useSelector(selectUserAddress);
  const dispath = useDispatch();

  const [OwshenWallet, setOwshenWallet] = useState({});
  const [destOwshenWallet, setDstOwshenWallet] = useState("");
  //TODO: its gonna be dynamic!
  const amount = 10_000_000_000_000_000_000;

  useEffect(() => {
    axios.get(`${coreEndpoint}/info`).then((result) => {
      setOwshenWallet({
        wallet: result.data.address,
        contract_address: result.data.contract_address,
        contract_abi: result.data.contract_abi,
        dive_address: result.data.dive_address,
        dive_abi: result.data.dive_abi,
      });
      dispath(
        setOwshen({
          wallet: result.data.address,
          contract_address: result.data.contract_address,
          contract_abi: result.data.contract_abi,
          dive_address: result.data.dive_address,
          dive_abi: result.data.dive_abi,
        })
      );
    });
  }, [OwshenWallet.wallet, OwshenWallet.contract_address, coreEndpoint]);

  useEffect(() => {
    const coinsIntervalId = setInterval(() => {
      axios.get(`${coreEndpoint}/coins`).then((result) => {
        dispath(setReceivedCoins(result.data.coins));
      });
    }, 5000);
    return () => clearInterval(coinsIntervalId);
  }, []);

  const { approve, allowance } = useApprove(
    OwshenWallet.dive_address,
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

        try {
          if (allowance < amount) await approve(utils.fromWei(amount, "wei"));
          const tx = await contract.deposit(
            pubKey,
            ephemeral,
            OwshenWallet.dive_address,
            utils.toBigInt(amount),
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

  return (
    <>
      <div className="header">
        <Web3ModalComponent />
      </div>
      <div className="modal" id="send-modal">
        <div className="modal-content">
          <div className="close-btn-container">
            <h3>Send</h3>
            <div style={{ cursor: "pointer" }} onClick={closeSend}>
              X
            </div>
          </div>
          <p>Privately send ERC-20 tokens to Owshen users!</p>
          <br />
          <div>
            <label>
              <b>From: </b>
            </label>
            <select>
              <option>Your Ethereum Account</option>
              <option>Your Owshen Account</option>
            </select>
          </div>
          <div>
            <label>
              <b>To: </b>
            </label>
            <input
              onChange={(e) => setDstOwshenWallet(e.target.value)}
              type="text"
            />
          </div>
          <button
            onClick={async () => await getStealth()}
            style={{ fontSize: "1em" }}
          >
            Send
          </button>
        </div>
      </div>
      <div style={{ width: "100vw", textAlign: "center" }}>
        <div style={{ fontSize: "xx-large", marginTop: "45vh" }}>
          🌊 Owshen Wallet 🌊
          <br />
          <i style={{ fontSize: "0.5em" }}>
            <b>Address: {OwshenWallet.wallet}</b>
          </i>
          <br />
          <i style={{ fontSize: "0.5em" }}>
            <b>Balance:</b> 0.0 ETH | 0.0 DIVE
          </i>
          <br />
          <button onClick={openSend}>Send</button>
        </div>
      </div>
      <ReceivedCoinList />
    </>
  );
};

export default Main;
