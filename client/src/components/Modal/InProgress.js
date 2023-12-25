import { useState } from "react";
import Modal from "./Modal";
import ReactLoading from "react-loading";

const InProgress = ({ isOpen, setIsOpen }) => {
  return (
    <Modal title="In progress..." isOpen={isOpen} setIsOpen={setIsOpen}>
      <h1 className="text-2xl mt-16 mb-10">Under development. </h1>

      <ReactLoading
        className="mx-auto"
        type="spin"
        color="green"
        height={100}
        width={100}
      />
    </Modal>
  );
};

export default InProgress;
