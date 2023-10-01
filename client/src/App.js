import React, { useEffect, useState } from "react";

import "./styles/main.css";
import { openSend } from "./utils/helper";
import { send } from "./utils/http";
import axios from "axios";

const App = () => {
  const [OwshenWallet, setOwshenWallet] = useState("");
  useEffect(() => {
    axios.get("/info").then((result) => setOwshenWallet(result));
  }, [OwshenWallet]);
  return (
    <>
      <div className="modal" id="send-modal">
        <div className="modal-content">
          <h3>Send</h3>
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
            <input type="text" />
          </div>
          <button style={{ fontSize: "1em" }} onClick={send}>
            Send
          </button>
        </div>
      </div>
      <div style={{ width: "100vw", textAlign: "center" }}>
        <div style={{ fontSize: "xx-large", marginTop: "45vh" }}>
          🌊 Owshen Wallet 🌊
          <br />
          <i style={{ fontSize: "0.5em" }}>
            <b>Address: {OwshenWallet}</b>
          </i>
          <br />
          <i style={{ fontSize: "0.5em" }}>
            <b>Balance:</b> 0.0 ETH | 0.0 DIVE
          </i>
          <br />
          <button onClick={openSend}>Send</button>
        </div>
      </div>
    </>
  );
};

export default App;
