import Modal from ".";
import ReactLoading from "react-loading";

const InProgress = ({ isOpen, setIsOpen }) => {
  return (
    <Modal title="" isOpen={isOpen} setIsOpen={setIsOpen}>
      <div className="h-[300px] bg-[url('../pics/inProgress.png')] bg-contain bg-center">
      <ReactLoading
        className="mx-auto pt-24"
        type="spin"
        color="#2c21ff99"
        height={100}
        width={100}
      /></div>
    </Modal>
  );
};

export default InProgress;
