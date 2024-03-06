import React, { useEffect, useState, useRef } from "react";
import "./SignalGraph.scss";

var PIXEL_RATIO = (function () {
  var ctx = document.createElement("canvas").getContext("2d"),
    dpr = window.devicePixelRatio || 1,
    bsr =
      ctx.webkitBackingStorePixelRatio ||
      ctx.mozBackingStorePixelRatio ||
      ctx.msBackingStorePixelRatio ||
      ctx.oBackingStorePixelRatio ||
      ctx.backingStorePixelRatio ||
      1;

  return dpr / bsr;
})();

function SignalGraph({ endpoint, start_time, end_time, data }) {
  const canvasRef = useRef(null);
  // let [data, setData] = useState(null);
  let [canvasSize, setCanvasSize] = useState({ width: 0, height: 0 });
  let [message, setMessage] = useState("idle");
  let [zoom, setZoom] = useState(0.3);

  useEffect(() => {
    let canvas = canvasRef.current;
    canvas.width = canvas.clientWidth * 2;
    canvas.height = canvas.clientHeight * 2;
    setCanvasSize({ width: canvas.width, height: canvas.height });
  }, []);

  // Drawing
  useEffect(() => {
    let canvas = canvasRef.current;
    let ctx = canvas.getContext("2d", { alpha: false });
    let w = canvasSize.width;
    let h = canvasSize.height;

    let draw = function () {
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      if (data) {
        // Draw the signal
        let start = 0;
        let end = zoom;

        let sample_rate = data["sample_rate"];
        let samples = (end - start) * sample_rate;
        let pixel_step = w / samples;

        ctx.beginPath();

        if (data["flags"]["has_gps_fix"]) {
          ctx.strokeStyle = "#0f0";
        } else {
          ctx.strokeStyle = "#ffc145";
        }

        if (data["flags"]["is_clipping"]) {
          ctx.strokeStyle = "#f00";
        }

        ctx.lineWidth = 2;

        for (let i = 0; i < data["data"].length; i += 1) {
          if (i === 0) {
            ctx.moveTo(i * pixel_step, h / 2 - (data["data"][i] * h) / 2);
          }
          ctx.lineTo(i * pixel_step, h / 2 - (data["data"][i] * h) / 2);
        }
        ctx.stroke();

        // Draw end in bottom right
        ctx.fillStyle = "#aaa";
        ctx.font = "20px Space Mono, monospace";
        let fx = w - 20;
        let fy = h - 20;
        let text = `${(end * 1000).toFixed()}ms`;
        ctx.fillText(text, fx - (text.length * 20) / 1.5, fy);
        ctx.fillRect(fx, fy + 5, 1, -40);

        ctx.font = "20px Space Mono, monospace";
        fx = w / 2 + 10;
        fy = h - 20;
        ctx.fillText(`${((end / 2) * 1000).toFixed()}ms`, fx, fy);
        ctx.fillRect(fx - 10, fy + 5, 1, -40);

        ctx.font = "20px Space Mono, monospace";
        fx = 20;
        fy = h - 20;
        ctx.fillText(`0ms`, fx + 2, fy - 4);
        ctx.fillRect(fx - 10, fy + 5, 1, -40);
        // ctx.fillRect(fx - 10, fy + 5, 40, 1);

        ctx.font = "20px Space Mono, monospace";
        fx = 20;
        fy = 40;
        ctx.fillText(`1`, fx - 5, fy - 4);
        ctx.fillRect(fx - 10, fy - 25, 40, 1);
      }
    };

    draw();

    return () => {
      let ctx = canvas.getContext("2d");
      ctx.clearRect(0, 0, canvas.width, canvas.height);
    };
  }, [data, canvasSize, zoom]);

  useEffect(() => {
    let canvas = canvasRef.current;
    let onwheel = (e) => {
      e.preventDefault();

      let newZoom = zoom;

      newZoom += e.deltaY / 1000;

      if (newZoom < 0.01) newZoom = 0.01;
      if (newZoom > 0.36) newZoom = 0.36;

      if (newZoom === zoom) return;

      setZoom(newZoom);
    };

    canvas.addEventListener("wheel", onwheel);

    return () => {
      canvas.removeEventListener("wheel", onwheel);
    };
  }, [zoom]);

  let error = null;

  return (
    <div className="signal-graph">
      <div className="signal-axis signal-axis-x">
        <p>Capture interval (ms)</p>
      </div>
      <div className="signal-axis signal-axis-y">
        <p>Normalized voltage</p>
      </div>
      <canvas ref={canvasRef} className="signal-canvas" />
      {data == null ? (
        <div className="signal-error">
          <p>NO DATA</p>
        </div>
      ) : null}
    </div>
  );
}

export default SignalGraph;
