import React from "react";
import fMark from "../../assets/flowspeech-f-mark.png";

const FlowSpeechLogo = ({
  width = 120,
  className,
}: {
  width?: number;
  className?: string;
}) => {
  const iconSize = Math.round(width * 0.28);
  const fontSize = Math.round(width * 0.11);

  return (
    <div
      className={`flex flex-col items-center gap-2 ${className || ""}`}
      style={{ width }}
    >
      <img
        src={fMark}
        width={iconSize}
        height={iconSize}
        alt=""
        style={{ objectFit: "contain" }}
      />
      <span
        className="font-semibold tracking-tight whitespace-nowrap"
        style={{ fontSize }}
      >
        FlowSpeech
      </span>
    </div>
  );
};

export default FlowSpeechLogo;
