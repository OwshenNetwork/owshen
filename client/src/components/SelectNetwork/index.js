import React, { useState, useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import { selectIsTest } from "../../store/containerSlice";
import Dropdown from "../DropDown";
import { setNetworkDetails } from "../../store/containerSlice";
import {
  getNetworkNameByChainId,
  isChainIdExist,
  networkDetails,
} from "../../utils/networkDetails";
import { useSelectNetworkApi } from "../../api/hooks/useSelectNetworkApi";
import { chainIdOfWallet } from "../../utils/helper";
import { toast } from "react-toastify";

const SelectNetwork = () => {
  const dispatch = useDispatch();
  const [chainId, setChainId] = useState(null);
  const [network, setNetWork] = useState("Select Network");
  const isTest = useSelector(selectIsTest);
  const { setChainIdOnCore } = useSelectNetworkApi();

  const netWorkOptions = [
    {
      title: "Sepolia",
      value: "Sepolia",
    },
  ];

  useEffect(() => {
    if (!isTest) {
      netWorkOptions.push(
        {
          title: "Goerli",
          value: "Goerli",
        },
        { title: "Localhost", value: "Localhost" }
      );
    }
  });

  useEffect(() => {
    const getChainId = async () => {
      let ChainId = await chainIdOfWallet(); // Get the chainId
      if (!isChainIdExist(ChainId)) {
        return toast.error("please select your network");
      }

      const networkName = getNetworkNameByChainId(ChainId);

      setNetWork(networkName);
      const selectedNetwork = networkDetails[networkName];
      updateNetworkDetails(
        selectedNetwork.name,
        selectedNetwork.chainId,
        selectedNetwork.contractName
      );
      setChainId(ChainId);
    }; // Add any dependencies here if getChainId depends on props or state
    getChainId();
  });

  const updateNetworkDetails = (name, chainId, contractName) => {
    dispatch(setNetworkDetails({ name, chainId, contractName }));
  };
  const handelChangeNetwork = async (val) => {
    if (val) {
      const selectedNetwork = await networkDetails[val];
      setChainIdOnCore(selectedNetwork?.chainId, val, chainId);
    }
  };
  return (
    <>
      <Dropdown
        label={network}
        options={netWorkOptions}
        select={setNetWork}
        onChange={handelChangeNetwork}
        style={{
          color: "white",
          padding: "3px",
          borderRadius: "xl",
          backgroundColor: "blue-100",
          borderColor: "0",
          ":dark": {
            backgroundColor: "blue-900",
            borderColor: "gray-300",
          },
        }}
      />
    </>
  );
};

export default React.memo(SelectNetwork);
