import { useState, useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import { selectIsTest, selectNetwork } from "../../store/containerSlice";
import Dropdown from "../DropDown";
import { setNetworkDetails } from "../../store/containerSlice";
import {
  getNetworkNameByChainId,
  isChainIdExist,
  networkDetails,
} from "../../utils/networkDetails";
import { useSelectNetworkApi } from "../../api/hooks/useSelectNetworkApi";
import { chainIdOfWallet, SwitchNetwork } from "../../utils/helper";
import { toast } from "react-toastify";

const SelectNetwork = () => {
  const dispatch = useDispatch();
  const selectedNetwork = useSelector(selectNetwork);
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
  const getChainId = async () => {
    let ChainId = await chainIdOfWallet(); // Get the chainId
    if (!isChainIdExist(ChainId)) {
      return toast.error("please select your network")
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
  };
  useEffect(() => {
    getChainId();
  }, []);

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
        style={"!text-white !py-3 !rounded-xl !bg-blue-100 dark:!bg-blue-900 !border-0 dark:!border-gray-300"}
      />
    </>
  );
};

export default SelectNetwork;
