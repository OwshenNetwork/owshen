import axios from "axios";

export const send = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "none";
  axios
    .get("http://127.0.0.1:8000/withdraw")
    .then((result) => {
      alert(JSON.stringify(result));
    })
    .catch((error) => console.log(error));

  axios
    .get(
      "http://127.0.0.1:8000/stealth?address=OoOo322687fd00e98b776230992ecaab658128c19d9e3f4a095b41fdff6d99f6846c1"
    )
    .then((result) => {
      alert(JSON.stringify(result));
    })
    .catch((error) => console.log(error));
};
