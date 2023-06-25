import { useEffect } from "react";

const Main = () => {
  const socket = new WebSocket("ws://localhost:8000");

  socket.addEventListener("open", () => {
    console.log("sending");
    setTimeout(() => {
      socket.send("a".repeat(20000));
    }, 5000);
  });
  socket.addEventListener("message", (data) => {
    console.log(data);
  });
  socket.addEventListener("close", () => {});

  return <h1>MainView</h1>;
};

export default Main;
