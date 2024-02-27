import React, { useState, useRef, useEffect } from "react";
import MenuIcon from "../../pics/icons/HamburgerIcon.png";

const Drawer = ({ children }) => {
  const drawerRef = useRef(null);
  const [isOpen, setIsOpen] = useState(false);
  const [showOverlay, setShowOverlay] = useState(false); // New state variable for the overlay

  const toggleDrawer = () => {
    setIsOpen(!isOpen);
    setShowOverlay(!isOpen); // Toggle the overlay when the drawer opens or closes
  };

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (drawerRef.current && !drawerRef.current.contains(event.target)) {
        setIsOpen(false);
        setShowOverlay(false); // Hide the overlay when clicking outside the drawer
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  return (
    <>
      <button onClick={toggleDrawer}>
        <img className="w-7 dark:invert" src={MenuIcon} />
      </button>

      {showOverlay && <div className="fixed inset-0 bg-black opacity-50"></div>}

      <div
        ref={drawerRef}
        className={`fixed top-0 z-30 left-0 h-screen w-64 bg-white dark:bg-indigo-950 transform transition-transform ease-in-out duration-200 p-4 ${
          isOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        {/* Drawer content goes here */}
        <button
          className="mb-5 text-right text-red-600 w-full"
          onClick={toggleDrawer}
        >
          x
        </button>
        {children}
      </div>
    </>
  );
};

export default Drawer;
