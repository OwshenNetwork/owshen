import React, { useEffect } from "react";
import Web3ModalComponent from "../WalletConnection";
import Drawer from "../Drawer";
import MoonIcon from "../../pics/moon.png";
import SunIcon from "../../pics/sun.png";
import { Tooltip } from "react-tooltip";
import { useMediaQuery } from 'react-responsive';


const Header = ({ isDarkTheme, setIsDarkTheme }) => {
  useEffect(() => {
    // Update the localStorage value when the state changes
    localStorage.setItem("theme", isDarkTheme);
  }, [isDarkTheme]);

  const darkThemeHandler = () => {
    const newTheme = !isDarkTheme;
    setIsDarkTheme(newTheme);
    localStorage.setItem("theme", newTheme);
  };
  useEffect(() => {
    if (isDarkTheme) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [isDarkTheme]);

  const darkThemButton = (
    <button
      data-tooltip-id="Theme"
      onClick={darkThemeHandler}
      className="dark:invert ml-5"
    >
      <img
        className=" w-8"
        src={isDarkTheme ? SunIcon : MoonIcon}
        alt="theme Icon"
      />
    </button>
  );
  const isLargeScreen = useMediaQuery({ query: "(min-width: 1024px)" });

  return (
    <div className="header justify-between flex w-full  items-center">
      <div className="flex w-4/6 items-center justify-start">
        <Tooltip id="Theme" place="bottom" content="Change Theme" />

        {/* <img src={Logo} className="w-9 h-5 lg:w-[70px] lg:h-10" /> */}
        <h1 className="font-bold text-2xl lg:text-5xl pl-2 lg:pl-4">Owshen</h1>
      </div>
      {isLargeScreen ? (
        <div className="hidden lg:flex w-3/6 justify-end ml-auto">
          <Web3ModalComponent />
          {darkThemButton}
        </div>
      ) : (
        <div className="lg:hidden flex w-3/6 justify-end mr-auto">
          <Drawer>
            {darkThemButton}
            <Web3ModalComponent />
          </Drawer>
        </div>
      )}
    </div>
  );
};

export default Header;
