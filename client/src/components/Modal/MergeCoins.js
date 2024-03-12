import { useState } from "react";
import Modal from ".";
const MergeCoins = () => {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <>
      <Modal isOpen={isOpen} setIsOpen={setIsOpen}>
      <div className="my-10">
        <p>do you want to merge all of your coins</p>
      </div>

      </Modal>
      <button onClick={() => setIsOpen(true)}>click it</button>
    </>
  );
};

export default MergeCoins;
