import Modal from ".";

const InProgress = ({ isOpen, setIsOpen }) => {
  return (
    <Modal title="" isOpen={isOpen} setIsOpen={setIsOpen}>
      <div className="h-[300px] bg-[url('../pics/inProgress.png')] bg-contain bg-center"></div>
    </Modal>
  );
};

export default InProgress;
