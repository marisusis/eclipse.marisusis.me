import React from "react";
import SignalGraph from "./SignalGraph";
import { useEffect } from "react";
import "./SignalPanel.scss";

export default function SignalPanel({ config }) {
  const [data, setData] = React.useState(null);

  // Data fetching
  useEffect(() => {
    let fetching = false;

    // setTimeout(() => {}, Math.random() * 3000);

    let interval = setInterval(() => {
      if (fetching) {
        return;
      }

      let controller = new AbortController();
      const signal = controller.signal;

      setTimeout(() => {
        controller.abort();
        fetching = false;
      }, 4000);
      fetching = true;
      fetch(config.endpoint, { signal: signal })
        .then((response) => {
          fetching = false;
          if (!response.ok) {
            return Promise.reject(response);
          }

          return response.json();
        })
        .then((data) => {
          setData(data);
        })
        .catch((error) => {
          setData(null);
          fetching = false;
        });
    }, 1500);
    return () => {
      clearInterval(interval);
    };
  }, [config]);

  let status = <p></p>;

  if (data == null) {
    status = (
      <p className={data == null ? "offline" : "online"}>
        {data == null ? "OFFLINE" : "ONLINE"}
      </p>
    );
  } else {
    if (data["flags"]["has_gps_fix"]) {
      status = <p className="online">ONLINE</p>;
    } else {
      status = <p className="no-fix">NO GPS</p>;
    }
  }

  return (
    <div className="signal-panel">
      <div className="signal-info">
        <div className="signal-panel-title">
          <h2>{config.node_id}</h2>
          {status}
          {/* {data == null ? <p className="offline">OFFLINE</p> : null} */}
        </div>
        <p>{config.location ? config.location : "Earth"}</p>
      </div>
      <SignalGraph data={data} />
    </div>
  );
}
