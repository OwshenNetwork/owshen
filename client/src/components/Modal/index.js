import { useRef, useEffect } from "react";
import BackArrow from "../../pics/icons/arrow.png";

const Modal = ({ title, setIsOpen, isOpen, children }) => {
  const ref = useRef(null);

  // Close Modal when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (ref.current && !ref.current.contains(event.target)) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [ref, setIsOpen]);
  return (
    <div
      className={`${
        isOpen ? "block" : "hidden"
      } absolute z-10 left-0 top-0 w-full h-full backdrop:blur-md bg-[#0000005c] transition-opacity duration-300 ease-in-out `}
    >
      <div className="flex items-center h-full pb-48">
        <div
          ref={ref}
          className=" border-2 text-center md:w-3/4 lg:!min-w-[510px] lg:w-1/4 p-5 mt-16 mx-auto bg-white dark:bg-indigo-950 rounded-xl"
        >
          <div className="relative">
            <div
              className="cursor-pointer w-9 absolute top-2 dark:invert"
              onClick={() => setIsOpen(!isOpen)}
            >
              <img src={BackArrow} alt="BackArrow" />
            </div>
            <h3 className="font-bold text-3xl">{title}</h3>
          </div>
          {children}
        </div>
      </div>
    </div>
  );
};

export default Modal;
