/*
 * Ono-Sendai Rust Reading Course — progressive enhancement.
 *
 * Every page is complete without this script. It adds: multiple-choice checking, hint ordering,
 * annotation <-> code-line linking, expand/collapse and wrap controls for code, the mobile
 * navigation panel, and optional local progress (completed lessons, last lesson, hints,
 * checklists, private notes) with a reset.
 *
 * Rules: no network APIs, no dynamic code loading, no inline handlers, no HTML strings built from
 * data (text is always set via textContent / value). All storage access is wrapped; when storage
 * is unavailable the course keeps working with in-memory state and says so.
 * The markup contract is documented in docs/frontend.md.
 */
(function () {
  "use strict";

  var PREFIX = "ono-rrc:";
  var doc = document;
  doc.documentElement.classList.add("js");

  /* ---------- Storage ---------- */

  var memory = {};
  var storage = (function () {
    try {
      var s = window.localStorage;
      var probe = PREFIX + "probe";
      s.setItem(probe, "1");
      s.removeItem(probe);
      return s;
    } catch (e) {
      return null;
    }
  })();

  function load(key) {
    if (storage) {
      try {
        return storage.getItem(PREFIX + key);
      } catch (e) { /* fall through to memory */ }
    }
    return Object.prototype.hasOwnProperty.call(memory, key) ? memory[key] : null;
  }

  function save(key, value) {
    memory[key] = value;
    if (storage) {
      try {
        storage.setItem(PREFIX + key, value);
        return true;
      } catch (e) {
        return false;
      }
    }
    return false;
  }

  function remove(key) {
    delete memory[key];
    if (storage) {
      try { storage.removeItem(PREFIX + key); } catch (e) { /* ignore */ }
    }
  }

  function loadList(key) {
    try {
      var v = JSON.parse(load(key) || "[]");
      return Array.isArray(v) ? v.map(String) : [];
    } catch (e) {
      return [];
    }
  }

  function saveList(key, list) {
    return save(key, JSON.stringify(list));
  }

  function clearAll() {
    memory = {};
    if (!storage) return true;
    try {
      var keys = [];
      for (var i = 0; i < storage.length; i++) {
        var k = storage.key(i);
        if (k && k.indexOf(PREFIX) === 0) keys.push(k);
      }
      keys.forEach(function (k) { storage.removeItem(k); });
      return true;
    } catch (e) {
      return false;
    }
  }

  function all(sel, root) {
    return Array.prototype.slice.call((root || doc).querySelectorAll(sel));
  }

  function showStorageNote() {
    if (storage) return;
    all(".storage-note").forEach(function (n) { n.hidden = false; });
  }

  /* ---------- Navigation panel (narrow screens) ---------- */

  function initNav() {
    var toggle = doc.querySelector(".nav-toggle");
    var nav = doc.getElementById("course-nav");
    if (!toggle || !nav) return;
    var close = nav.querySelector(".nav-close");
    var wide = window.matchMedia ? window.matchMedia("(min-width: 1024px)") : null;
    var outside = [doc.querySelector("main"), doc.querySelector(".site-footer"), doc.querySelector(".site-header"), doc.querySelector(".skip-link")];
    toggle.hidden = false;
    if (close) close.hidden = false;

    function setInert(on) {
      outside.forEach(function (el) {
        if (!el) return;
        if (on) el.setAttribute("inert", ""); else el.removeAttribute("inert");
      });
    }

    function open() {
      nav.classList.add("is-open");
      doc.body.classList.add("nav-open");
      toggle.setAttribute("aria-expanded", "true");
      setInert(true);
      var current = nav.querySelector("[aria-current='page']");
      if (current) {
        var det = current.closest("details");
        if (det) det.open = true;
      }
      (close || current || nav).focus();
    }

    function shut(restoreFocus) {
      if (!nav.classList.contains("is-open")) return;
      nav.classList.remove("is-open");
      doc.body.classList.remove("nav-open");
      toggle.setAttribute("aria-expanded", "false");
      setInert(false);
      if (restoreFocus) toggle.focus();
    }

    toggle.addEventListener("click", function () {
      if (nav.classList.contains("is-open")) shut(true); else open();
    });
    if (close) close.addEventListener("click", function () { shut(true); });
    doc.addEventListener("keydown", function (e) {
      if ((e.key === "Escape" || e.key === "Esc") && nav.classList.contains("is-open")) {
        e.preventDefault();
        shut(true);
      }
    });
    nav.addEventListener("click", function (e) {
      var a = e.target.closest ? e.target.closest("a") : null;
      if (a) shut(false);
    });
    function onWide() {
      if (wide && wide.matches) shut(false);
    }
    if (wide) {
      if (wide.addEventListener) wide.addEventListener("change", onWide);
      else if (wide.addListener) wide.addListener(onWide);
    }
  }

  /* ---------- Progress ---------- */

  function doneMark() {
    var s = doc.createElement("span");
    s.className = "done-mark";
    s.textContent = "Completed";
    return s;
  }

  function renderDone() {
    var done = loadList("done");
    all("[data-lesson-id]").forEach(function (li) {
      var id = li.getAttribute("data-lesson-id");
      var isDone = done.indexOf(id) !== -1;
      li.classList.toggle("is-done", isDone);
      var link = li.querySelector("a");
      if (!link) return;
      var mark = link.querySelector(".done-mark");
      if (isDone && !mark) link.appendChild(doneMark());
      if (!isDone && mark) mark.parentNode.removeChild(mark);
    });
    var page = doc.body.getAttribute("data-page");
    all(".progress-summary").forEach(function (p) {
      var items = all("main [data-lesson-id]");
      var count = items.filter(function (li) { return li.classList.contains("is-done"); }).length;
      if (!items.length || count === 0) { p.hidden = true; return; }
      p.textContent = "You have completed " + count + " of " + items.length + " lessons" +
        (page === "chapter" ? " in this chapter." : " in the course.");
      p.hidden = false;
    });
  }

  function initLessonProgress() {
    var lesson = doc.body.getAttribute("data-lesson");
    if (lesson) save("last", lesson);
    var box = doc.querySelector(".complete");
    var btn = box ? box.querySelector(".mark-complete") : null;
    if (!btn) return;
    var status = box.querySelector(".complete-status");
    box.hidden = false;
    function update(announce) {
      var done = loadList("done").indexOf(lesson) !== -1;
      btn.setAttribute("aria-pressed", done ? "true" : "false");
      btn.textContent = done ? "Completed — mark as not complete" : "Mark lesson complete";
      if (announce && status) {
        status.textContent = done
          ? (storage ? "Lesson marked as complete." : "Lesson marked as complete for this visit (this browser cannot save it).")
          : "Lesson marked as not complete.";
      }
    }
    btn.addEventListener("click", function () {
      var done = loadList("done");
      var i = done.indexOf(lesson);
      if (i === -1) done.push(lesson); else done.splice(i, 1);
      saveList("done", done);
      update(true);
      renderDone();
    });
    update(false);
  }

  function initContinue() {
    var box = doc.querySelector(".continue");
    if (!box) return;
    var last = load("last");
    if (!last) return;
    var item = null;
    all("main [data-lesson-id]").forEach(function (li) {
      if (!item && li.getAttribute("data-lesson-id") === last) item = li;
    });
    var src = item ? item.querySelector("a") : null;
    if (!src) return;
    var title = src.querySelector(".l-title");
    var a = doc.createElement("a");
    a.className = "btn";
    a.href = src.getAttribute("href");
    a.textContent = "Continue where you left off: " + (title ? title.textContent : src.textContent);
    box.textContent = "";
    box.appendChild(a);
    box.hidden = false;
  }

  /* ---------- Multiple choice ---------- */

  function initMultipleChoice() {
    all("form.mc").forEach(function (form) {
      var check = form.querySelector(".mc-check");
      var answer = form.querySelector(".mc-answer");
      var verdict = form.querySelector(".mc-verdict");
      if (!check || !verdict) return;
      check.hidden = false;
      if (answer) answer.hidden = true;
      form.addEventListener("submit", function (e) {
        e.preventDefault();
        var chosen = form.querySelector("input[type=radio]:checked");
        all(".choice", form).forEach(function (c) { c.classList.remove("is-correct", "is-incorrect"); });
        all(".mc-explanation", form).forEach(function (x) { x.hidden = true; });
        verdict.classList.remove("is-correct", "is-incorrect");
        if (!chosen) {
          verdict.textContent = "Choose an answer first.";
          return;
        }
        var correct = chosen.getAttribute("data-correct") === "true";
        var label = chosen.closest(".choice");
        if (label) label.classList.add(correct ? "is-correct" : "is-incorrect");
        verdict.classList.add(correct ? "is-correct" : "is-incorrect");
        verdict.textContent = correct
          ? "Correct."
          : "Not correct. Read the explanation, then try again.";
        var fb = form.querySelector('.mc-explanation[data-choice="' + chosen.value + '"]');
        if (fb) fb.hidden = false;
        if (correct && answer) answer.hidden = false;
      });
      form.addEventListener("change", function () {
        verdict.textContent = "";
        verdict.classList.remove("is-correct", "is-incorrect");
      });
    });
  }

  /* ---------- Hints: Hint N stays locked until Hint N-1 was opened ---------- */

  function initHints() {
    all(".exercise").forEach(function (ex) {
      var id = ex.getAttribute("data-exercise");
      var hints = all("details.hint", ex);
      if (!hints.length) return;
      var key = "hints:" + id;
      var opened = parseInt(load(key) || "0", 10) || 0;

      function refresh() {
        hints.forEach(function (h) {
          var level = parseInt(h.getAttribute("data-level"), 10) || 1;
          var locked = level > opened + 1 && !h.open;
          var summary = h.querySelector("summary");
          var lock = h.querySelector(".hint-lock");
          h.classList.toggle("is-locked", locked);
          if (lock) lock.hidden = !locked;
          if (summary) {
            if (locked) summary.setAttribute("aria-disabled", "true");
            else summary.removeAttribute("aria-disabled");
          }
        });
      }

      hints.forEach(function (h) {
        var level = parseInt(h.getAttribute("data-level"), 10) || 1;
        var summary = h.querySelector("summary");
        if (summary) {
          summary.addEventListener("click", function (e) {
            if (h.classList.contains("is-locked")) e.preventDefault();
          });
        }
        h.addEventListener("toggle", function () {
          if (h.open && level > opened + 1) {
            h.open = false;
            return;
          }
          if (h.open && level > opened) {
            opened = level;
            save(key, String(opened));
          }
          refresh();
        });
      });
      refresh();
    });
  }

  /* ---------- Code figures: annotations, expand all, wrap ---------- */

  function parseLines(spec) {
    var parts = String(spec || "").split("-");
    var a = parseInt(parts[0], 10);
    var b = parseInt(parts[1] || parts[0], 10);
    return isNaN(a) ? null : [a, isNaN(b) ? a : b];
  }

  function initCodeFigures() {
    all(".code-figure").forEach(function (fig) {
      var tools = fig.querySelector(".code-tools");
      if (tools) tools.hidden = false;
      var lines = all(".line[data-line]", fig);
      var anns = all("details.annotation", fig);

      function refreshActive() {
        var ranges = anns.filter(function (d) { return d.open; }).map(function (d) { return parseLines(d.getAttribute("data-lines")); });
        lines.forEach(function (line) {
          var n = parseInt(line.getAttribute("data-line"), 10);
          var active = ranges.some(function (r) { return r && n >= r[0] && n <= r[1]; });
          line.classList.toggle("is-active", active);
        });
        all(".ann-marker", fig).forEach(function (m) {
          var d = fig.querySelector('details.annotation[data-ann="' + m.getAttribute("data-ann") + '"]');
          m.setAttribute("aria-expanded", d && d.open ? "true" : "false");
        });
        var btn = fig.querySelector(".ann-all");
        if (btn) {
          var allOpen = anns.length > 0 && anns.every(function (d) { return d.open; });
          btn.textContent = allOpen ? "Collapse all annotations" : "Expand all annotations";
          btn.setAttribute("data-state", allOpen ? "expanded" : "collapsed");
        }
      }

      anns.forEach(function (d) { d.addEventListener("toggle", refreshActive); });

      all(".ann-marker", fig).forEach(function (m) {
        m.addEventListener("click", function (e) {
          var d = fig.querySelector('details.annotation[data-ann="' + m.getAttribute("data-ann") + '"]');
          if (!d) return;
          e.preventDefault();
          d.open = true;
          var s = d.querySelector("summary");
          if (s) {
            s.focus({ preventScroll: true });
            if (s.scrollIntoView) s.scrollIntoView({ block: "nearest" });
          }
        });
      });

      var allBtn = fig.querySelector(".ann-all");
      if (allBtn) {
        allBtn.addEventListener("click", function () {
          var openAll = allBtn.getAttribute("data-state") !== "expanded";
          anns.forEach(function (d) { d.open = openAll; });
          refreshActive();
        });
      }

      var wrap = fig.querySelector(".code-wrap");
      if (wrap) {
        wrap.addEventListener("click", function () {
          var on = !fig.classList.contains("wrap");
          fig.classList.toggle("wrap", on);
          wrap.setAttribute("aria-pressed", on ? "true" : "false");
        });
      }
      refreshActive();
    });
  }

  /* ---------- Checklists and scratchpads ---------- */

  function initChecklists() {
    all("fieldset.checklist").forEach(function (fs) {
      var key = "check:" + fs.getAttribute("data-exercise");
      var ticked = loadList(key);
      var boxes = all("input[type=checkbox]", fs);
      boxes.forEach(function (b) { b.checked = ticked.indexOf(b.value) !== -1; });
      fs.addEventListener("change", function () {
        saveList(key, boxes.filter(function (b) { return b.checked; }).map(function (b) { return b.value; }));
      });
    });
  }

  function initNotes() {
    all("textarea.notes").forEach(function (ta) {
      var key = "notes:" + ta.getAttribute("data-exercise");
      var status = ta.parentNode.querySelector(".notes-status");
      var stored = load(key);
      if (stored !== null) ta.value = stored;
      var timer = null;
      function persist() {
        timer = null;
        var ok = ta.value === "" ? (remove(key), true) : save(key, ta.value);
        if (status) {
          status.textContent = ok
            ? "Saved in this browser only."
            : "Not saved: this browser does not allow the course to store notes.";
        }
      }
      ta.addEventListener("input", function () {
        if (timer) window.clearTimeout(timer);
        timer = window.setTimeout(persist, 400);
      });
      ta.addEventListener("blur", function () {
        if (timer) { window.clearTimeout(timer); persist(); }
      });
    });
  }

  /* ---------- Reset (About page) ---------- */

  function initReset() {
    var box = doc.querySelector(".reset");
    if (!box) return;
    var btn = box.querySelector(".reset-progress");
    var confirmBox = box.querySelector(".reset-confirm");
    var yes = box.querySelector(".reset-yes");
    var no = box.querySelector(".reset-no");
    var status = box.querySelector(".reset-status");
    if (!btn || !confirmBox || !yes || !no) return;
    box.hidden = false;
    btn.addEventListener("click", function () {
      confirmBox.hidden = false;
      btn.setAttribute("aria-expanded", "true");
      if (status) status.textContent = "";
      no.focus();
    });
    no.addEventListener("click", function () {
      confirmBox.hidden = true;
      btn.setAttribute("aria-expanded", "false");
      btn.focus();
    });
    yes.addEventListener("click", function () {
      var ok = clearAll();
      confirmBox.hidden = true;
      btn.setAttribute("aria-expanded", "false");
      if (status) {
        status.textContent = ok
          ? "Your local progress for this course was reset."
          : "The browser did not allow the stored progress to be removed.";
      }
      btn.focus();
      renderDone();
    });
  }

  function init() {
    showStorageNote();
    initNav();
    initLessonProgress();
    renderDone();
    initContinue();
    initMultipleChoice();
    initHints();
    initCodeFigures();
    initChecklists();
    initNotes();
    initReset();
  }

  if (doc.readyState === "loading") doc.addEventListener("DOMContentLoaded", init);
  else init();
})();
