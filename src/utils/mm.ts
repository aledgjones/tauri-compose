const getMM = () => {
  if (!document.body) return 3.78; // fallback for 96 DPI
  const div = document.createElement("div");
  div.style.position = "fixed";
  div.style.width = "1mm";
  document.body.appendChild(div);
  const width = div.clientWidth;
  document.body.removeChild(div);
  return width || 3.78; // fallback if clientWidth is 0
};

export const pxPerMm = getMM();
