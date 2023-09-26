const send = () => {
  let elem = document.getElementById("send-modal");
  elem.style.display = "none";
  $.getJSON("/withdraw", function (data) {
    alert(JSON.stringify(data));
  });
};
