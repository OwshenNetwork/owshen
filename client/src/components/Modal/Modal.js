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
  }, [ref]);
  return (
    <div
      className={`${
        isOpen ? "block" : "hidden"
      } fixed z-10 left-0 top-0 w-full h-full overflow-auto backdrop:blur-md bg-[#0000005c] `}
    >
      <div
        ref={ref}
        className=" border-2 text-center w-1/4 p-5 my-[10%] mx-auto bg-white rounded-xl"
      >
        <div className="relative">
          <div
            className="cursor-pointer w-9 absolute top-2"
            onClick={() => setIsOpen(!isOpen)}
          >
            <img src={BackArrow} />
          </div>
          <h3 className="font-bold text-3xl">{title}</h3>
        </div>
        {children}
      </div>
    </div>
  );
};

export default Modal;
