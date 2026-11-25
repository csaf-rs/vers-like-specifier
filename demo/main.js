import init, { Vls, Comparator, Containment } from "./pkg/vers_like_specifier.js";

await init();

document.querySelectorAll("button:disabled").forEach((button) => (button.disabled = false));

const parseForm = document.getElementById("parse-form");
const parseOut = document.getElementById("parse-output");
const parseErrorOut = document.getElementById("parse-error");

parseForm.addEventListener("submit", (event) => {
  event.preventDefault();
  parseOut.replaceChildren();
  parseErrorOut.textContent = "";

  try {
    const vls = Vls.parse(parseForm["parse-vls"].value);
    const list = document.createElement("ul");
    for (const constraint of vls.constraints) {
      const item = document.createElement("li");
      item.textContent = `comparator: ${Comparator[constraint.comparator]}, version: "${constraint.version}"`;
      list.append(item);
    }
    parseOut.append(`Parsed: "${vls}" (${vls.constraints.length} constraint(s))`, list);
  } catch (e) {
    parseErrorOut.textContent = `${e}`;
  }
});

const checkForm = document.getElementById("check-form");
const checkOut = document.getElementById("check-output");
const checkErrorOut = document.getElementById("check-error");

checkForm.addEventListener("submit", (event) => {
  event.preventDefault();
  checkOut.textContent = "";
  checkErrorOut.textContent = "";

  try {
    const vls = Vls.parse(checkForm["check-vls"].value);
    checkOut.textContent = Containment[vls.contains(checkForm["check-version"].value)];
  } catch (e) {
    checkErrorOut.textContent = `${e}`;
  }
});
