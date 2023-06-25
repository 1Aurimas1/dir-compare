import { useEffect } from "react";
//import io from "socket.io-client";
import { Socket } from "engine.io-client";

const Main = () => {
  const socket = new Socket("localhost:8000", { transports: ["websocket"] });
  socket.on("open", () => {
    console.log("sending");
    setTimeout(() => {
      //socket.send("4yeet");
      //socket.send("a".repeat(10));
      socket.send("a".repeat(20000));
    }, 5000);
    socket.on("message", (data) => {
      console.log(data);
    });
    socket.on("close", () => {});
  });
  //const socketURL = "localhost:8000";
  ////useEffect(() => {
  //const socket = io(socketURL, {
  //  transports: ["websocket"],
  //});

  //socket.on("connect_error", (e) => {
  //  console.log("connect error: ", e);
  //});
  ////socket.connect();

  //socket.on("connect", () => {
  //  console.log("Connect event");
  //});
  //socket.on("connected", () => {
  //  console.log("Connected event");
  //});

  //socket.on("data", (message) => {
  //  console.log(message);
  //});
  //socket.on("message", (message) => {
  //  console.log(message);
  //});

  //socket.on("disconnect", () => {
  //  console.log("Disconnected event");
  //});
  //setTimeout(() => { socket.send("ayaya"); }, 5000);

  //return () => {
  //  console.log("Unmounted");
  //  socket.disconnect();
  //};
  //}, []);
  return <h1>ayaya</h1>;
};

export default Main;
