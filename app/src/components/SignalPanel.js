import React from "react";
import SignalGraph from "./SignalGraph";
import { useEffect } from "react";
import "./SignalPanel.scss";

export default function SignalPanel({ node_id, location, data, status }) {
  let status_elem = <p></p>;
  switch (status) {
    case "offline":
      status_elem = <p className="offline">OFFLINE</p>;
      break;
    case "online":
      status_elem = <p className="online">ONLINE</p>;
      break;
    case "no-fix":
      status_elem = <p className="no-fix">NO SYNC</p>;
      break;
  }

  return (
    <div className="signal-panel">
      <div className="signal-info">
        <div className="signal-panel-title">
          <h2>{node_id}</h2>
          {status_elem}
        </div>
        <p>{location}</p>
      </div>
      <SignalGraph data={data} />
    </div>
  );
}
