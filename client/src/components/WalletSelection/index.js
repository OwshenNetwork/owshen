import { useEffect } from "react";
import { Link } from "react-router-dom";
import Logo from "../../pics/logo.png";
import { useMainApi } from "../../api/hooks/useMainApi";
import { selectIsOwshenWalletExist } from "../../store/containerSlice";
import { useNavigate } from "react-router-dom";
import { useSelector } from "react-redux";
const WalletSelection = () => {
  const IsOwshenWalletExist = useSelector(selectIsOwshenWalletExist);
  const navigate = useNavigate();
  const { getInfo } = useMainApi();

  useEffect(() => {
    getInfo();
    if (IsOwshenWalletExist) {
      navigate("/");
    }
  });
  const btnCS =
    "border lg:w-[270px] w-full rounded-xl p-3  my-2  ease-in-out duration-300 flex items-center justify-around bg-[#EBEDEF]  hover:bg-[#BBDCFBCC] dark:bg-indigo-950";
  return (
    <>
      {<div>IsOwshenWalletExist</div>}
      <div className="text-center flex h-full w-full justify-center items-center ">
        <div className="w-[470px] border-2 p-6 rounded-lg">
          <h3 className="text-2xl my-7">Welcome</h3>
          <img src={Logo} alt="logo" className="w-32 mx-auto" />
          <div className="flex flex-col items-center mt-4">
            <Link className={`${btnCS}`} to="/walletSelection/importWallet">
              Import Owshen Wallet
            </Link>
            <Link className={btnCS} to="/walletSelection/createNewWallet">
              Create New Owshen Wallet
            </Link>
          </div>
        </div>
      </div>
    </>
  );
};

export default WalletSelection;
