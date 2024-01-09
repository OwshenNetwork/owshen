import { useState, useEffect } from "react";
import { useSelector } from "react-redux";
import axios from "axios";
import { useAccount, useEnsName } from "wagmi";
import { selectIsTest } from "../../store/containerSlice";
import Dropdown from "../DropDown";

const SelectNetwork = () => {
  const coreEndpoint =
    process.env.REACT_APP_OWSHEN_ENDPOINT || "http://127.0.0.1:9000";
  const { chainId } = useAccount();
  const [network, setNetWork] = useState("select network");

  useEffect(() => {}, []);

  const isTest = useSelector(selectIsTest);
  const netWorkOptions = [
    {
      title: "goerli",
      value: "goerli",
    },
    !isTest ? { title: "localhost", value: "localhost" } : null,
  ];
  useEffect(() => {
    checkNetwork(chainId);
  }, [chainId]);

  const checkNetwork = async (val) => {
    switch (val) {
      case 5:
        setNetWork("goerli");
        break;
      case 1337:
        setNetWork("localhost");
        break;
      default:
        setNetWork("select network");
    }
  };
  const handelChangeNetwork = async (val) => {
    console.log(val);
    switch (val) {
      case "goerli":
        setChainId(5);
        break;
      case "localhost":
        setChainId(1337);
        break;
      default:
        setNetWork("select network");
    }
  };

  const setChainId = async (chainId) => {
    let chain_id = chainId;
    if (chainId === 5) {
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
