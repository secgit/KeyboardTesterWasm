(function () {
  const heldKeysList = document.getElementById("held-keys");
  const noHeldKeysLabel = document.getElementById("no-held-keys");
  const eventLogBody = document.getElementById("event-log");
  const patternSequenceEl = document.getElementById("pattern-sequence");
  const patternCountsEl = document.getElementById("pattern-counts");
  const clearButton = document.getElementById("clear-log");
  const pauseToggle = document.getElementById("toggle-pause");

  const activeKeys = new Map();
  const repeatCounts = new Map();
  const patternBuffer = [];

  const MAX_LOG_ROWS = 300;
  const MAX_PATTERN_LENGTH = 80;

  let lastEventTime = null;
  let originTime = performance.now();

  function formatSeconds(timestamp) {
    return ((timestamp - originTime) / 1000).toFixed(3);
  }

  function formatDelta(timestamp) {
    if (lastEventTime == null) {
      return "—";
    }
    return Math.round(timestamp - lastEventTime).toString();
  }

  function renderHeldKeys() {
    heldKeysList.innerHTML = "";
    if (activeKeys.size === 0) {
      noHeldKeysLabel.hidden = false;
      return;
    }

    noHeldKeysLabel.hidden = true;
    const fragment = document.createDocumentFragment();
    for (const keyInfo of activeKeys.values()) {
      const li = document.createElement("li");
      const keySpan = document.createElement("span");
      keySpan.className = "pill-key";
      keySpan.textContent = keyInfo.key;

      const metaSpan = document.createElement("span");
      metaSpan.className = "pill-meta";
      metaSpan.textContent = keyInfo.code;

      li.append(keySpan, metaSpan);
      fragment.appendChild(li);
    }
    heldKeysList.appendChild(fragment);
  }

  function renderPatternSequence() {
    if (patternBuffer.length === 0) {
      patternSequenceEl.textContent = "Waiting for repeated events...";
      return;
    }

    const sequence = patternBuffer.map((entry) => entry.label).join(" → ");
    patternSequenceEl.textContent = sequence;
  }

  function renderPatternCounts() {
    patternCountsEl.innerHTML = "";

    if (repeatCounts.size === 0) {
      return;
    }

    const items = [...repeatCounts.entries()].sort(
      (a, b) => b[1].count - a[1].count
    );
    const fragment = document.createDocumentFragment();

    for (const [code, data] of items) {
      const { count, key } = data;
      const li = document.createElement("li");

      const keySpan = document.createElement("span");
      keySpan.className = "pill-key";
      keySpan.textContent = key;

      const metaSpan = document.createElement("span");
      metaSpan.className = "pill-meta";
      metaSpan.textContent = `${code} ×${count}`;

      li.append(keySpan, metaSpan);
      fragment.appendChild(li);
    }

    patternCountsEl.appendChild(fragment);
  }

  function appendLogRow({ type, key, code, repeat, timeStamp }) {
    const row = document.createElement("tr");
    const delta = formatDelta(timeStamp);
    const deltaDisplay = delta === "—" ? delta : `${delta} ms`;
    row.innerHTML = `
      <td>${formatSeconds(timeStamp)}</td>
      <td>${deltaDisplay}</td>
      <td>${type}</td>
      <td>${key}</td>
      <td>${code}</td>
      <td>${repeat ? "yes" : "no"}</td>
    `;
    eventLogBody.prepend(row);

    while (eventLogBody.children.length > MAX_LOG_ROWS) {
      eventLogBody.removeChild(eventLogBody.lastChild);
    }
  }

  function handleKeyDown(event) {
    if (pauseToggle.checked) {
      return;
    }

    const { key, code, repeat, timeStamp } = event;

    if (!activeKeys.has(code)) {
      activeKeys.set(code, { key, code, pressedAt: timeStamp });
      renderHeldKeys();
    }

    appendLogRow({ type: "keydown", key, code, repeat, timeStamp });
    lastEventTime = timeStamp;

    if (repeat) {
      const label = key.length === 1 ? key : code;
      patternBuffer.push({ label, timeStamp });
      if (patternBuffer.length > MAX_PATTERN_LENGTH) {
        patternBuffer.shift();
      }

      const existing = repeatCounts.get(code) || { count: 0, key };
      repeatCounts.set(code, { count: existing.count + 1, key });
      renderPatternSequence();
      renderPatternCounts();
    }
  }

  function handleKeyUp(event) {
    if (pauseToggle.checked) {
      return;
    }

    const { key, code, timeStamp } = event;

    appendLogRow({ type: "keyup", key, code, repeat: false, timeStamp });
    lastEventTime = timeStamp;

    if (activeKeys.has(code)) {
      activeKeys.delete(code);
      renderHeldKeys();
    }
  }

  function resetAll() {
    activeKeys.clear();
    repeatCounts.clear();
    patternBuffer.length = 0;
    eventLogBody.innerHTML = "";
    lastEventTime = null;
    originTime = performance.now();
    renderHeldKeys();
    renderPatternSequence();
    renderPatternCounts();
  }

  clearButton.addEventListener("click", () => {
    resetAll();
    pauseToggle.checked = false;
  });

  document.body.addEventListener("click", () => {
    document.body.focus();
  });

  document.addEventListener("keydown", handleKeyDown, true);
  document.addEventListener("keyup", handleKeyUp, true);

  document.body.setAttribute("tabindex", "0");
  document.body.focus();

  renderHeldKeys();
  renderPatternSequence();
})();
