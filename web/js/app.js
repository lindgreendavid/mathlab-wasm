import init, { solve_json } from "../pkg/mathlab_wasm.js";

const elements = {
  form: document.querySelector("#controls"),
  functionSelect: document.querySelector("#function"),
  method: document.querySelector("#method"),
  first: document.querySelector("#first"),
  second: document.querySelector("#second"),
  secondGroup: document.querySelector("#second-group"),
  firstHelp: document.querySelector("#first-help"),
  secondHelp: document.querySelector("#second-help"),
  tolerance: document.querySelector("#tolerance"),
  maxIterations: document.querySelector("#max-iterations"),
  run: document.querySelector("#run"),
  wasmState: document.querySelector("#wasm-state"),
  status: document.querySelector("#result-status"),
  message: document.querySelector("#result-message"),
  root: document.querySelector("#metric-root"),
  residual: document.querySelector("#metric-residual"),
  iterations: document.querySelector("#metric-iterations"),
  evaluations: document.querySelector("#metric-evaluations"),
  chart: document.querySelector("#trace-chart"),
  chartDescription: document.querySelector("#trace-desc"),
  chartNote: document.querySelector("#chart-note"),
  traceBody: document.querySelector("#trace-body"),
};

const presets = {
  cubic: { bisection: [1, 2], newton: [1.5, 0], secant: [1, 2] },
  cosine: { bisection: [0, 1], newton: [1, 0], secant: [0, 1] },
  repeated: { bisection: [0, 2], newton: [2, 0], secant: [0, 2] },
  "newton-cycle": { bisection: [-2, -1], newton: [0, 0], secant: [-2, -1] },
  flat: { bisection: [-1, 1], newton: [0.5, 0], secant: [-1, 0.5] },
};

const statusLabels = {
  converged: "Converged",
  "invalid-bracket": "Invalid bracket",
  "zero-derivative": "Derivative too small",
  "collapsed-secant": "Secant collapsed",
  "non-finite": "Non-finite iterate",
  "cycle-detected": "Cycle detected",
  "max-iterations": "Iteration limit reached",
};

let engineReady = false;
let latestResult = null;
let chartScale = "residual";

function formatNumber(value) {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  if (value === 0) return "0";
  const magnitude = Math.abs(value);
  if (magnitude >= 10000 || magnitude < 0.0001) return value.toExponential(5);
  return value.toPrecision(8).replace(/\.?0+$/, "");
}

function updateInputMeaning(applyPreset = true) {
  const method = elements.method.value;
  const functionId = elements.functionSelect.value;
  const usesSecond = method !== "newton";
  elements.secondGroup.hidden = !usesSecond;
  elements.second.disabled = !usesSecond;
  elements.firstHelp.textContent = method === "bisection" ? "Left bracket endpoint" : method === "newton" ? "Initial iterate x₀" : "First secant iterate x₀";
  elements.secondHelp.textContent = method === "bisection" ? "Right bracket endpoint" : "Second secant iterate x₁";
  if (applyPreset) {
    const [first, second] = presets[functionId][method];
    elements.first.value = String(first);
    elements.second.value = String(second);
  }
}

function clearChart() {
  while (elements.chart.firstElementChild && !["title", "desc"].includes(elements.chart.firstElementChild.tagName.toLowerCase())) {
    elements.chart.firstElementChild.remove();
  }
  [...elements.chart.children].filter((node) => !["title", "desc"].includes(node.tagName.toLowerCase())).forEach((node) => node.remove());
}

function svgElement(name, attributes = {}) {
  const element = document.createElementNS("http://www.w3.org/2000/svg", name);
  Object.entries(attributes).forEach(([key, value]) => element.setAttribute(key, String(value)));
  return element;
}

function renderChart(trace) {
  clearChart();
  if (!trace.length) {
    elements.chartDescription.textContent = "No numerical trace exists for this rejected input.";
    return;
  }
  const width = 760;
  const height = 370;
  const margin = { top: 28, right: 28, bottom: 52, left: 82 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;
  const values = trace.map((step) => chartScale === "residual" ? Math.log10(Math.max(Math.abs(step.fx), Number.EPSILON)) : step.x);
  let minimum = Math.min(...values);
  let maximum = Math.max(...values);
  if (minimum === maximum) { minimum -= 1; maximum += 1; }
  const padding = (maximum - minimum) * 0.12;
  minimum -= padding;
  maximum += padding;
  const x = (index) => margin.left + (trace.length === 1 ? innerWidth / 2 : index / (trace.length - 1) * innerWidth);
  const y = (value) => margin.top + (maximum - value) / (maximum - minimum) * innerHeight;

  for (let tick = 0; tick <= 4; tick += 1) {
    const value = minimum + (maximum - minimum) * tick / 4;
    const yPosition = y(value);
    elements.chart.append(svgElement("line", { x1: margin.left, x2: width - margin.right, y1: yPosition, y2: yPosition, class: "guide" }));
    const label = svgElement("text", { x: margin.left - 12, y: yPosition + 4, "text-anchor": "end" });
    label.textContent = chartScale === "residual" ? `10^${value.toFixed(1)}` : formatNumber(value);
    elements.chart.append(label);
  }
  elements.chart.append(svgElement("line", { x1: margin.left, x2: margin.left, y1: margin.top, y2: height - margin.bottom, class: "axis" }));
  elements.chart.append(svgElement("line", { x1: margin.left, x2: width - margin.right, y1: height - margin.bottom, y2: height - margin.bottom, class: "axis" }));

  const points = values.map((value, index) => `${x(index)},${y(value)}`).join(" ");
  elements.chart.append(svgElement("polyline", { points, class: "trace-line" }));
  values.forEach((value, index) => {
    elements.chart.append(svgElement("circle", { cx: x(index), cy: y(value), r: 5, class: "trace-point" }));
    if (trace.length <= 12 || index === 0 || index === trace.length - 1) {
      const label = svgElement("text", { x: x(index), y: height - margin.bottom + 24, "text-anchor": "middle" });
      label.textContent = String(trace[index].iteration);
      elements.chart.append(label);
    }
  });
  const xTitle = svgElement("text", { x: margin.left + innerWidth / 2, y: height - 12, "text-anchor": "middle" });
  xTitle.textContent = "Iteration n";
  elements.chart.append(xTitle);
  const yTitle = svgElement("text", { x: 18, y: margin.top + innerHeight / 2, transform: `rotate(-90 18 ${margin.top + innerHeight / 2})`, "text-anchor": "middle" });
  yTitle.textContent = chartScale === "residual" ? "Absolute residual |f(x)|" : "Estimate x";
  elements.chart.append(yTitle);
  elements.chartDescription.textContent = `${trace.length} recorded iterates showing ${chartScale === "residual" ? "absolute residual on a logarithmic scale" : "the estimated root position"}.`;
  elements.chartNote.textContent = chartScale === "residual" ? "Residual uses a logarithmic vertical scale; zero is shown at machine epsilon." : "Position shows the raw binary64 iterate returned by Rust.";
}

function renderTable(trace) {
  if (!trace.length) {
    elements.traceBody.innerHTML = '<tr><td colspan="5">The method rejected the inputs before an iterate was created.</td></tr>';
    return;
  }
  elements.traceBody.replaceChildren(...trace.map((step) => {
    const row = document.createElement("tr");
    [step.iteration, formatNumber(step.x), formatNumber(step.fx), formatNumber(step.step_size), formatNumber(step.bracket_width)].forEach((value) => {
      const cell = document.createElement("td");
      cell.textContent = String(value);
      row.append(cell);
    });
    return row;
  }));
}

function renderResult(result) {
  latestResult = result;
  elements.status.textContent = statusLabels[result.status] ?? result.status;
  elements.status.dataset.status = result.status;
  elements.message.textContent = result.message;
  elements.root.textContent = formatNumber(result.root);
  elements.residual.textContent = formatNumber(result.residual);
  elements.iterations.textContent = String(result.iterations);
  elements.evaluations.textContent = String(result.function_evaluations);
  renderChart(result.trace);
  renderTable(result.trace);
}

function runExperiment() {
  if (!engineReady) return;
  const first = Number(elements.first.value);
  const second = Number(elements.second.value);
  const tolerance = Number(elements.tolerance.value);
  const maxIterations = Number(elements.maxIterations.value);
  if (![first, second, tolerance, maxIterations].every(Number.isFinite)) {
    elements.message.textContent = "Enter finite numeric values before running the method.";
    return;
  }
  try {
    const payload = solve_json(elements.method.value, elements.functionSelect.value, first, second, tolerance, maxIterations);
    renderResult(JSON.parse(payload));
  } catch (error) {
    elements.status.textContent = "Input error";
    elements.message.textContent = error instanceof Error ? error.message : String(error);
  }
}

elements.form.addEventListener("submit", (event) => { event.preventDefault(); runExperiment(); });
elements.method.addEventListener("change", () => { updateInputMeaning(true); runExperiment(); });
elements.functionSelect.addEventListener("change", () => { updateInputMeaning(true); runExperiment(); });
document.querySelectorAll("[data-scale]").forEach((button) => {
  button.addEventListener("click", () => {
    chartScale = button.dataset.scale;
    document.querySelectorAll("[data-scale]").forEach((candidate) => candidate.setAttribute("aria-pressed", String(candidate === button)));
    if (latestResult) renderChart(latestResult.trace);
  });
});

document.querySelectorAll("[data-case]").forEach((button) => {
  button.addEventListener("click", () => {
    const caseId = button.dataset.case;
    if (caseId === "newton-cycle") {
      elements.functionSelect.value = "newton-cycle";
      elements.method.value = "newton";
    } else if (caseId === "repeated-bisection") {
      elements.functionSelect.value = "repeated";
      elements.method.value = "bisection";
    } else {
      elements.functionSelect.value = "repeated";
      elements.method.value = "secant";
    }
    updateInputMeaning(true);
    runExperiment();
    document.querySelector("#laboratory").scrollIntoView({ behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
  });
});

updateInputMeaning(true);
elements.run.disabled = true;

try {
  await init();
  engineReady = true;
  elements.run.disabled = false;
  elements.wasmState.textContent = "Rust/WebAssembly engine ready · no server computation";
  runExperiment();
} catch (error) {
  elements.wasmState.textContent = "The WebAssembly engine could not load. Serve the built web directory over HTTP.";
  elements.message.textContent = error instanceof Error ? error.message : String(error);
}
