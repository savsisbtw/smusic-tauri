function setCoverArt(url) {
  const a = document.getElementById("cover-a");
  const b = document.getElementById("cover-b");
  const front = a.classList.contains("is-front") ? a : b;
  const back = front === a ? b : a;

  back.src = url || "";
  back.classList.add("is-front");
  front.classList.remove("is-front");
}

function setPlayIconState(isPlaying) {
  const btn = document.getElementById("btn-play");
  if (isPlaying) {
    btn.classList.add("is-playing");
  } else {
    btn.classList.remove("is-playing");
  }
}

function staggerIn(elements, baseDelayMs, stepMs, maxStaggered) {
  elements.forEach((el, i) => {
    if (i < maxStaggered) {
      el.style.animationDelay = baseDelayMs + i * stepMs + "ms";
    } else {
      el.style.animationDelay = "0ms";
    }
  });
}

const IDLE_LINES = ["ничего не играет", "выбери трек в библиотеке", "тишина"];

let idleTypewriterTimer = null;

function startIdleTypewriter() {
  const el = document.getElementById("idle-typewriter");
  if (!el) return;
  stopIdleTypewriter();

  let lineIndex = 0;
  let charIndex = 0;
  let deleting = false;

  function tick() {
    const current = IDLE_LINES[lineIndex];
    if (!deleting) {
      charIndex++;
      el.textContent = current.slice(0, charIndex);
      if (charIndex === current.length) {
        idleTypewriterTimer = setTimeout(() => {
          deleting = true;
          tick();
        }, 1600);
        return;
      }
      idleTypewriterTimer = setTimeout(tick, 55);
    } else {
      charIndex--;
      el.textContent = current.slice(0, charIndex);
      if (charIndex === 0) {
        deleting = false;
        lineIndex = (lineIndex + 1) % IDLE_LINES.length;
        idleTypewriterTimer = setTimeout(tick, 400);
        return;
      }
      idleTypewriterTimer = setTimeout(tick, 30);
    }
  }

  tick();
}

function stopIdleTypewriter() {
  if (idleTypewriterTimer) {
    clearTimeout(idleTypewriterTimer);
    idleTypewriterTimer = null;
  }
}
