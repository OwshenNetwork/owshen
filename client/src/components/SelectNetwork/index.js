import { useState, useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import axios from "axios";
import { useAccount } from "wagmi";
import { selectIsTest, selectNetwork } from "../../store/containerSlice";
import Dropdown from "../DropDown";
import { toast } from "react-toastify";
import { setNetworkDetails } from "../../store/containerSlice";

const SelectNetwork = () => {
  const dispatch = useDispatch();
  const coreEndpoint = process.env.REACT_APP_OWSHEN_ENDPOINT || "";
  const accountData = useAccount();
  const selectedNetwork = useSelector(selectNetwork);
  const chainId = accountData ? accountData.chainId : undefined;
  const [network, setNetWork] = useState("Select Network");

  useEffect(() => {}, []);

  const isTest = useSelector(selectIsTest);
  const netWorkOptions = [
    isTest
      ? {
          title: "Goerli",
          value: "Goerli",
        }
      : {},
    !isTest && {
      title: "Sepolia",
      value: "Sepolia",
    },
    isTest ? { title: "Localhost", value: "Localhost" } : {},
  ];
  useEffect(() => {
    checkNetwork(chainId);
  }, [chainId]);

  const checkNetwork = async (val) => {
    switch (val) {
      case 5:
        setNetWork("Goerli");
        updateNetworkDetails("Goerli", 5, "Goerli");
        break;
      case 1337:
        setNetWork("Localhost");
        updateNetworkDetails("Localhost", 1337, "Localhost");
        break;
      case 11155111:
        setNetWork("Sepolia");
        updateNetworkDetails("Sepolia", 11155111, "Ethereum_Sepolia");
        break;
      case 5556:
        setNetWork("Local-Testnet");
        updateNetworkDetails("testnet", 5556, "Local-Testnet");
        break;
      default:
        setNetWork("Select Network");
    }
  };
  const handelChangeNetwork = async (val) => {
    switch (val) {
      case "Goerli":
        setChainId(5, val);
        break;
      case "Localhost":
        setChainId(1337, val);
        break;
      case "Sepolia":
        setChainId(11155111, val);
        break;
      case "Local-Testnet":
        setChainId(5556, val);
        break;
      default:
        setNetWork(network);
    }
  };
  const updateNetworkDetails = (name, chainId, contractName) => {
    dispatch(setNetworkDetails({ name, chainId, contractName }));
  };
  const setChainId = async (newChainId, val) => {
    if (newChainId !== chainId && val) {
      toast.error(`Please change your wallet network to ${val}`);
    }
    updateNetworkDetails(val, newChainId);
    let chain_id = newChainId;
    if (newChainId === 5) {
      chain_id = "0x5";
    }
    await axios
      .post(`${coreEndpoint}/set-network`, null, {
        params: { chain_id },
      })
      .then((response) => {
        console.log("Response:", response.data);
      })
      .catch((error) => {
        console.error("Error:", error);
      });
  };
  useEffect(() => {
    if (window.ethereum) {
      window.ethereum.on("chainChanged", (data) => {
        let chain_id = parseInt(data, 16);
        checkNetwork(chain_id);
        handelChangeNetwork(selectedNetwork.name);
        setChainId(chain_id.toString(), selectedNetwork.name);
      });
    }
  }, []);
  return (
    <>
      {!isTest && (
        <Dropdown
          label={network}
          options={netWorkOptions}
          select={setNetWork}
          onChange={handelChangeNetwork}
          style="!bg-gray-200 !text-white !py-3 !rounded-xl border-0 "
        />
      )}
    </>
  );
};

export default SelectNetwork;
