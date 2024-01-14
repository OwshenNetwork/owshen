import { useState, useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import axios from "axios";
import { useAccount } from "wagmi";
import { selectIsTest } from "../../store/containerSlice";
import Dropdown from "../DropDown";
import { toast } from "react-toastify";
import { setNetworkDetails } from "../../store/containerSlice";

const SelectNetwork = () => {
  const dispatch = useDispatch();
  const coreEndpoint =
    process.env.REACT_APP_OWSHEN_ENDPOINT || "http://127.0.0.1:9000";
  const accountData = useAccount();
  const chainId = accountData ? accountData.chainId : undefined;
  const [network, setNetWork] = useState("select network");

  useEffect(() => {}, []);

  const isTest = useSelector(selectIsTest);
  const netWorkOptions = [
    !isTest && {
      title: "goerli",
      value: "goerli",
    },
    {
      title: "sepolia",
      value: "sepolia",
    },
    !isTest && { title: "localhost", value: "localhost" },
  ];
  useEffect(() => {
    checkNetwork(chainId);
  }, [chainId]);

  const checkNetwork = async (val) => {
    switch (val) {
      case 5:
        setNetWork("goerli");
        updateNetworkDetails("goerli", 5, "goerli");
        break;
      case 1337:
        setNetWork("localhost");
        updateNetworkDetails("localhost", 1337, "localhost");
        break;
      case 11155111:
        setNetWork("sepolia");
        updateNetworkDetails("sepolia", 11155111, "ethereum_sepolia");
        break;
      default:
        setNetWork("select network");
    }
  };
  const handelChangeNetwork = async (val) => {
    switch (val) {
      case "goerli":
        setChainId(5, val);
        break;
      case "localhost":
        setChainId(1337, val);
        break;
      case "sepolia":
        setChainId(11155111, val);
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
      toast.error(`please change your wallet network to ${val}`);
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
      window.ethereum.on("chainChanged", () => {
        window.location.reload();
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
