"use strict";

const searchEl = document.querySelector("#search-form .search-input");

const suggestionsEl = document.createElement("div");
suggestionsEl.className = "suggestions";
suggestionsEl.hidden = true;

let suggestions = [];
let selectedIndex = -1;
let debounceTimer = null;
let controller = null;

if (searchEl) {
  searchEl.parentElement.appendChild(suggestionsEl);

  searchEl.addEventListener("input", () => {
    const query = searchEl.value.trim();

    clearTimeout(debounceTimer);

    if (!query) {
      if (controller) {
        controller.abort();
      }

      suggestions = [];
      hideSuggestions();
      return;
    }

    debounceTimer = setTimeout(() => {
      updateSuggestions(query);
    }, 0);
  });

  suggestionsEl.addEventListener("mousedown", (e) => {
    const item = e.target.closest(".suggestions-item");

    if (!item) {
      return;
    }

    searchEl.value = item.dataset.value;
    searchEl.focus();
    searchEl.form.submit();
  });

  searchEl.addEventListener("keydown", (e) => {
    if (suggestions.length === 0) {
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      const target = (selectedIndex + 1) % suggestions.length;
      setSelected(target);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const target =
        (selectedIndex - 1 + suggestions.length) % suggestions.length;
      setSelected(target);
    } else if (e.key === "Enter") {
      if (selectedIndex >= 0) {
        e.preventDefault();

        searchEl.value = suggestions[selectedIndex].value;
        searchEl.focus();
        searchEl.form.submit();
      }
    } else if (e.key === "Escape") {
      suggestions = [];
      renderSuggestions();
    }
  });

  searchEl.addEventListener("blur", () => {
    setTimeout(() => hideSuggestions(), 20);
  });

  searchEl.addEventListener("focus", () => {
    unhideSuggestions();
  });
}

async function updateSuggestions(query) {
  await fetchSuggestions(query);
  renderSuggestions();
}

function renderSuggestions() {
  selectedIndex = -1;
  suggestionsEl.innerHTML = "";

  if (suggestions.length === 0) {
    suggestionsEl.hidden = true;
    return;
  }

  suggestionsEl.hidden = false;

  suggestions.forEach((suggestion) => {
    if (suggestion.type !== "search") {
      return;
    }

    const itemEl = document.createElement("div");
    itemEl.className = "suggestions-item";
    itemEl.textContent = suggestion.value;
    itemEl.dataset.value = suggestion.value;

    suggestionsEl.appendChild(itemEl);
  });
}

async function fetchSuggestions(query) {
  if (controller) {
    controller.abort();
  }

  controller = new AbortController();

  try {
    const response = await fetch(`/complete?q=${encodeURIComponent(query)}`, {
      signal: controller.signal,
    });

    if (!response.ok) {
      return;
    }

    suggestions = await response.json();

    return suggestions;
  } catch (err) {
    if (err.name !== "AbortError") {
      console.error(err);
    }

    return [];
  }
}

function hideSuggestions() {
  suggestionsEl.hidden = true;
}

function unhideSuggestions() {
  if (suggestions.length === 0) {
    return;
  }
  suggestionsEl.hidden = false;
}

function setSelected(index) {
  const prev = suggestionsEl.children[selectedIndex];
  if (prev) {
    prev.classList.remove("selected");
  }

  selectedIndex = index;

  const toSet = suggestionsEl.children[selectedIndex];
  if (toSet) {
    toSet.classList.add("selected");
  }
}

function clearSelected() {
  let selectedEl = suggestionsEl.children[selectedIndex];
  if (selectedEl) {
    selectedEl.classList.remove("selected");
  }
  selectedIndex = -1;
}
