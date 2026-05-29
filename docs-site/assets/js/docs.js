(function () {
  const root = document.documentElement;
  const saved = localStorage.getItem("lds-docs-theme") || "light";
  root.dataset.theme = saved;

  function qs(selector) {
    return document.querySelector(selector);
  }

  document.querySelectorAll("[data-theme-toggle]").forEach((button) => {
    button.addEventListener("click", () => {
      const next = root.dataset.theme === "dark" ? "light" : "dark";
      root.dataset.theme = next;
      localStorage.setItem("lds-docs-theme", next);
    });
  });

  document.querySelectorAll("[data-menu-toggle]").forEach((button) => {
    button.addEventListener("click", () => qs(".sidebar")?.classList.toggle("open"));
  });

  document.querySelectorAll(".nav a").forEach((link) => {
    const current = location.pathname.split("/").pop() || "index.html";
    if (link.getAttribute("href") === current) {
      link.classList.add("active");
    }
  });

  document.querySelectorAll("[data-back-to-top]").forEach((button) => {
    button.addEventListener("click", () => scrollTo({ top: 0, behavior: "smooth" }));
  });

  const search = qs("[data-search]");
  const results = qs("[data-search-results]");
  if (search && results) {
    search.addEventListener("input", () => {
      const term = search.value.trim().toLowerCase();
      if (!term) {
        results.style.display = "none";
        results.innerHTML = "";
        return;
      }
      const pages = Array.from(document.querySelectorAll(".nav a"));
      const matches = pages.filter((link) => link.textContent.toLowerCase().includes(term));
      results.innerHTML = matches.length
        ? matches.map((link) => `<a href="${link.getAttribute("href")}">${link.textContent}</a>`).join("<br>")
        : "No matches";
      results.style.display = "block";
    });
  }
})();
