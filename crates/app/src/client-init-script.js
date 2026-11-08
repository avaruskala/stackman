window.MathJax = {
  tex: {
    inlineMath: [['\\(', '\\)'], ['$', '$']],
    displayMath: [['\\[', '\\]'], ['$$', '$$']],
    processEscapes: true,
    packages: { '[+]': ['ams', 'noerrors', 'noundefined'] }
  },
  options: {
    ignoreHtmlClass: 'tex2jax_ignore',
    processHtmlClass: 'tex2jax_process'
  },
  startup: {
    typeset: false
  }
};

// STACK-JS VLE side: sandboxed iframes talk to this page only via postMessage.
(function () {
  var IFRAMES = {};
  var INPUTS = {};
  var INPUTS_INPUT_EVENT = {};
  var VALIDATION_STATE_EVENT = {};
  var INPUTS_SCROLL_EVENT = {};
  var DISABLE_CHANGES = false;

  function questionArea(el, includeOutcome) {
    var iter = el;
    while (iter && !(iter.classList && (iter.classList.contains('formulation') ||
      (includeOutcome && iter.classList.contains('outcome'))))) {
      iter = iter.parentElement;
    }
    return iter || null;
  }

  function getIframe(id) {
    if (id in IFRAMES && IFRAMES[id].isConnected) {
      return IFRAMES[id];
    }
    var candidate = document.getElementById(id);
    if (candidate && candidate.nodeName.toLowerCase() === 'iframe') {
      IFRAMES[id] = candidate;
      return candidate;
    }
    return null;
  }

  function send(tgt, obj) {
    var frm = getIframe(tgt);
    if (!frm || !frm.contentWindow) {
      return;
    }
    obj.tgt = tgt;
    frm.contentWindow.postMessage(JSON.stringify(obj), '*');
  }

  function vleGetElement(id) {
    var candidate = document.getElementById(id);
    if (candidate && questionArea(candidate, true)) {
      return candidate;
    }
    return null;
  }

  function findInputIn(root, name) {
    if (!root) {
      return null;
    }
    var esc = CSS.escape(name);
    var selectors = [
      '.stack-input-control[data-input-name="' + esc + '"]:not(.stack-matrix-cell)',
      'input[id$="_' + esc + '"]',
      'textarea[id$="_' + esc + '"]',
      'input[id$="_' + esc + '_1"][type=radio]',
      'input[id$="_' + esc + '_1"][type=checkbox]',
      'select[id$="_' + esc + '"]',
      'input[id="' + esc + '"]'
    ];
    for (var i = 0; i < selectors.length; i++) {
      var found = root.querySelector(selectors[i]);
      if (found) {
        return found;
      }
    }
    return null;
  }

  function vleGetInputElement(name, srciframe, outside) {
    if (outside === undefined) {
      outside = true;
    }
    var frame = document.getElementById(srciframe);
    var area = frame ? questionArea(frame, true) : null;
    var found = findInputIn(area, name);
    if (found || !outside) {
      return found;
    }
    var areas = document.querySelectorAll('.formulation, .outcome');
    for (var i = 0; i < areas.length; i++) {
      found = findInputIn(areas[i], name);
      if (found) {
        return found;
      }
    }
    return null;
  }

  function groupOf(input) {
    if ((input.type === 'radio' || input.type === 'checkbox') && input.name) {
      var area = questionArea(input, true) || document;
      return area.querySelectorAll('input[name="' + CSS.escape(input.name) + '"]');
    }
    return [input];
  }

  function vleGetSubmitButton(srciframe) {
    var frame = document.getElementById(srciframe);
    var area = frame ? questionArea(frame, false) : null;
    if (!area) {
      return null;
    }
    return area.querySelector('.im-controls *[id$="-submit"][type=submit]');
  }

  function vleUpdateInput(el) {
    el.dispatchEvent(new Event('change'));
    el.dispatchEvent(new Event('input'));
    if (el.type === 'radio' || el.type === 'checkbox') {
      el.dispatchEvent(new Event('click'));
    }
  }

  function vleUpdateDom(el) {
    if (window.MathJax && window.MathJax.typesetPromise) {
      window.MathJax.typesetPromise([el]).catch(function () { });
    }
  }

  function scrollPosition(el) {
    var max = el.scrollHeight - el.clientHeight;
    return max <= 0 ? 0 : el.scrollTop / max;
  }

  function isEvilAttribute(name, value) {
    var lc = name.toLowerCase();
    if (lc.indexOf('on') === 0) {
      return true;
    }
    if (lc === 'src' || lc.endsWith('href')) {
      var v = value.replace(/\s+/g, '').toLowerCase();
      if (v.indexOf('javascript:') > -1 || v.indexOf('data:text') > -1) {
        return true;
      }
    }
    return false;
  }

  function sanitize(src) {
    var doc = new DOMParser().parseFromString(src, 'text/html');
    doc.querySelectorAll('script, style').forEach(function (el) { el.remove(); });
    doc.querySelectorAll('*').forEach(function (el) {
      Array.prototype.slice.call(el.attributes).forEach(function (a) {
        if (isEvilAttribute(a.name, a.value)) {
          el.removeAttribute(a.name);
        }
      });
    });
    return doc.body;
  }

  function inputValue(input) {
    if (input.type === 'checkbox') {
      return input.checked;
    }
    if (input.type === 'radio') {
      var val = '';
      groupOf(input).forEach(function (r) { if (r.checked) { val = r.value; } });
      return val;
    }
    return input.value;
  }

  var KEY_COUNTER = 0;
  function inputKey(input) {
    if (input.id) {
      return input.id;
    }
    if (!input.__stackjsKey) {
      KEY_COUNTER += 1;
      input.__stackjsKey = 'stackjs-input-' + KEY_COUNTER;
    }
    return input.__stackjsKey;
  }

  // Registers src as a listener of input; isNew tells whether DOM listeners must be bound.
  function addListener(map, input, src) {
    var key = inputKey(input);
    var isNew = !(key in map);
    if (isNew) {
      map[key] = [];
    }
    if (map[key].indexOf(src) === -1) {
      map[key].push(src);
    }
    var group = input.type === 'radio' ? groupOf(input) : [input];
    return { key: key, isNew: isNew, group: group };
  }

  window.addEventListener('message', function (e) {
    if (!(typeof e.data === 'string' || e.data instanceof String)) {
      return;
    }
    var msg = null;
    try {
      msg = JSON.parse(e.data);
    } catch (err) {
      return;
    }
    if (!msg || !msg.version || String(msg.version).indexOf('STACK-JS') !== 0) {
      return;
    }
    if (!('src' in msg) || !('type' in msg)) {
      return;
    }
    if (!getIframe(msg.src)) {
      return;
    }
    var response = { version: 'STACK-JS:1.6.0' };
    var input = null;
    var element = null;
    var reg = null;

    switch (msg.type) {
      case 'register-input-listener':
        input = vleGetInputElement(msg.name, msg.src, !msg['limit-to-question']);
        if (input === null) {
          send(msg.src, {
            version: response.version, type: 'error',
            msg: 'Failed to connect to input: "' + msg.name + '"'
          });
          return;
        }
        response.type = 'initial-input';
        response.name = msg.name;
        response.value = inputValue(input);
        var tag = input.nodeName.toLowerCase();
        if (tag === 'select') {
          response['input-type'] = 'select';
          response['input-readonly'] = input.hasAttribute('disabled');
        } else if (tag === 'textarea') {
          response['input-type'] = input.dataset.stackInputType === 'freetext' ? 'freetext' : 'textarea';
          response['input-readonly'] = input.hasAttribute('disabled');
        } else if (input.type === 'checkbox' || input.type === 'radio') {
          response['input-type'] = input.type;
          response['input-readonly'] = input.hasAttribute('disabled');
        } else {
          response['input-type'] = input.type;
          response['input-readonly'] = input.hasAttribute('readonly') || input.disabled;
        }
        response['input-dataset'] = {};
        for (var k in input.dataset) {
          response['input-dataset'][k] = input.dataset[k];
        }

        reg = addListener(INPUTS, input, msg.src);
        if (reg.isNew) {
          (function (key, group, name) {
            group.forEach(function (inp) {
              inp.addEventListener('change', function () {
                if (DISABLE_CHANGES) {
                  return;
                }
                if (inp.type === 'radio' && !inp.checked) {
                  return;
                }
                var resp = {
                  version: 'STACK-JS:1.0.0', type: 'changed-input',
                  name: name, value: inputValue(inp)
                };
                (INPUTS[key] || []).forEach(function (tgt) { send(tgt, resp); });
              });
            });
          })(reg.key, reg.group, msg.name);
        }

        if (msg['track-input'] && input.type !== 'radio') {
          var reg2 = addListener(INPUTS_INPUT_EVENT, input, msg.src);
          if (reg2.isNew) {
            (function (key, inp, name) {
              inp.addEventListener('input', function () {
                if (DISABLE_CHANGES) {
                  return;
                }
                var resp = {
                  version: 'STACK-JS:1.0.0', type: 'changed-input',
                  name: name, value: inputValue(inp)
                };
                (INPUTS_INPUT_EVENT[key] || []).forEach(function (tgt) { send(tgt, resp); });
              });
            })(reg2.key, input, msg.name);
          }
        }
        send(msg.src, response);
        break;

      case 'track-input-scroll':
        input = vleGetInputElement(msg.name, msg.src, !msg['limit-to-question']);
        if (input === null) {
          send(msg.src, {
            version: response.version, type: 'error',
            msg: 'Failed to connect to input: "' + msg.name + '"'
          });
          return;
        }
        reg = addListener(INPUTS_SCROLL_EVENT, input, msg.src);
        if (reg.isNew) {
          (function (key, inp, name) {
            inp.addEventListener('scroll', function () {
              var resp = {
                version: 'STACK-JS:1.6.0', type: 'input-scroll-position',
                name: name, position: scrollPosition(inp)
              };
              (INPUTS_SCROLL_EVENT[key] || []).forEach(function (tgt) { send(tgt, resp); });
            });
          })(reg.key, input, msg.name);
        }
        send(msg.src, {
          version: response.version, type: 'input-scroll-position',
          name: msg.name, position: scrollPosition(input)
        });
        break;

      case 'track-validation-state':
        input = vleGetInputElement(msg.name, msg.src, !msg['limit-to-question']);
        if (input === null) {
          send(msg.src, {
            version: response.version, type: 'error',
            msg: 'Failed to connect to input: "' + msg.name + '"'
          });
          return;
        }
        reg = addListener(VALIDATION_STATE_EVENT, input, msg.src);
        if (reg.isNew) {
          var target = (input.type === 'radio' || input.type === 'checkbox')
            ? (input.closest('.stack-choices-group') || input) : input;
          (function (key, name) {
            target.addEventListener('stack-validation', function (ev) {
              var resp = {
                version: 'STACK-JS:1.5.0', type: 'validation-state',
                name: name, valid: ev.detail.valid, completed: ev.detail.completed
              };
              (VALIDATION_STATE_EVENT[key] || []).forEach(function (tgt) { send(tgt, resp); });
            });
          })(reg.key, msg.name);
        }
        break;

      case 'changed-input':
        input = vleGetInputElement(msg.name, msg.src);
        if (input === null) {
          send(msg.src, {
            version: 'STACK-JS:1.0.0', type: 'error',
            msg: 'Failed to modify input: "' + msg.name + '"'
          });
          return;
        }
        DISABLE_CHANGES = true;
        if (input.type === 'checkbox') {
          input.checked = msg.value;
          vleUpdateInput(input);
        } else if (input.type === 'radio') {
          groupOf(input).forEach(function (r) {
            r.checked = r.value === String(msg.value);
            vleUpdateInput(r);
          });
        } else {
          input.value = msg.value;
          vleUpdateInput(input);
        }
        DISABLE_CHANGES = false;
        (INPUTS[inputKey(input)] || []).forEach(function (tgt) {
          if (tgt !== msg.src) {
            send(tgt, {
              version: response.version, type: 'changed-input',
              name: msg.name, value: msg.value
            });
          }
        });
        break;

      case 'clear-input':
        input = vleGetInputElement(msg.name, msg.src);
        if (input === null) {
          return;
        }
        if (input.nodeName.toLowerCase() === 'select') {
          input.value = '';
          vleUpdateInput(input);
        } else if (input.type === 'checkbox' || input.type === 'radio') {
          groupOf(input).forEach(function (inp) {
            inp.checked = input.type === 'radio' && inp.value === '';
            vleUpdateInput(inp);
          });
        } else if (input.value !== '') {
          input.value = '';
          vleUpdateInput(input);
        }
        break;

      case 'register-button-listener':
        element = vleGetElement(msg.target);
        if (element === null) {
          send(msg.src, {
            version: 'STACK-JS:1.2.0', type: 'error',
            msg: 'Failed to find element: "' + msg.target + '"'
          });
          return;
        }
        (function (src, target) {
          element.addEventListener('click', function (ev) {
            send(src, { version: 'STACK-JS:1.2.0', type: 'button-click', name: target });
            ev.preventDefault();
          });
        })(msg.src, msg.target);
        break;

      case 'toggle-visibility':
        element = vleGetElement(msg.target);
        if (element === null) {
          send(msg.src, {
            version: 'STACK-JS:1.0.0', type: 'error',
            msg: 'Failed to find element: "' + msg.target + '"'
          });
          return;
        }
        if (msg.set === 'show') {
          element.style.display = 'block';
          vleUpdateDom(element);
        } else if (msg.set === 'hide') {
          element.style.display = 'none';
        }
        break;

      case 'change-content':
        element = vleGetElement(msg.target);
        if (element === null) {
          send(msg.src, {
            version: response.version, type: 'error',
            msg: 'Failed to find element: "' + msg.target + '"'
          });
          return;
        }
        element.innerHTML = sanitize(msg.content).innerHTML;
        vleUpdateDom(element);
        break;

      case 'get-content':
        element = vleGetElement(msg.target);
        send(msg.src, {
          version: response.version, type: 'xfer-content',
          target: msg.target, content: element !== null ? element.innerHTML : null
        });
        break;

      case 'resize-frame':
        var frmEl = getIframe(msg.src);
        if (frmEl && frmEl.parentElement) {
          element = frmEl.parentElement;
          element.style.width = msg.width;
          element.style.height = msg.height;
          frmEl.style.width = '100%';
          frmEl.style.height = '100%';
        }
        break;

      case 'ping':
        send(msg.src, { version: response.version, type: 'ping' });
        return;

      case 'query-submit-button':
        input = vleGetSubmitButton(msg.src);
        send(msg.src, {
          version: response.version, type: 'submit-button-info',
          value: (input === null || input.hasAttribute('hidden')) ? null : input.value
        });
        return;

      case 'enable-submit-button':
      case 'relabel-submit-button':
        input = vleGetSubmitButton(msg.src);
        if (input === null) {
          send(msg.src, {
            version: response.version, type: 'error',
            msg: 'Could not find matching submit button for this question.'
          });
          return;
        }
        if (msg.type === 'enable-submit-button') {
          input.disabled = !msg.enabled;
        } else {
          input.textContent = msg.name;
          input.value = msg.name;
        }
        return;

      case 'submit-button-info':
      case 'initial-input':
      case 'error':
        break;

      default:
        send(msg.src, {
          version: response.version, type: 'error',
          msg: 'Unknown message-type: "' + msg.type + '"'
        });
    }
  });

  // Creates a sandboxed iframe inside the holder. allow-same-origin is never granted.
  window.stackmanCreateIframe = function (iframeid, content, holder, title, scrolling) {
    var frm = document.createElement('iframe');
    frm.id = iframeid;
    frm.style.width = '100%';
    frm.style.height = '100%';
    frm.style.border = '0';
    if (scrolling === false) {
      frm.setAttribute('scrolling', 'no');
      frm.style.overflow = 'hidden';
    } else {
      frm.setAttribute('scrolling', 'yes');
    }
    frm.title = title;
    frm.setAttribute('referrerpolicy', 'no-referrer');
    frm.setAttribute('sandbox', 'allow-scripts allow-downloads');
    frm.srcdoc = content;
    holder.replaceChildren(frm);
    IFRAMES[iframeid] = frm;
  };

  window.stackmanRegisterIframe = function (iframeid, frm) {
    IFRAMES[iframeid] = frm;
  };
})();

window.stackmanRenderPreview = function (fillCorrect) {
  setTimeout(function () {
    // 0. Register or mount STACK-JS iframes (all [[jsxgraph]] and [[iframe]] blocks run sandboxed in iframes)
    document.querySelectorAll('.stack-iframe-holder').forEach(function (el) {
      var frameId = el.getAttribute('data-frame-id');
      var frm = el.querySelector('iframe'); var srcdoc = el.getAttribute('data-frame-srcdoc');
      if (!frm && srcdoc && frameId) {
        window.stackmanCreateIframe(
          frameId,
          srcdoc,
          el,
          el.getAttribute('data-frame-title') || 'STACK IFRAME',
          el.getAttribute('data-frame-scrolling') !== 'false'
        );
      } else if (frm && frameId) {
        window.stackmanRegisterIframe(frameId, frm);
      }
    });

    // 1. Populate or clear STACK input controls when fillCorrect changes
    var controls = document.querySelectorAll('.stack-input-control');
    controls.forEach(function (ctrl) {
      var type = (ctrl.getAttribute('type') || '').toLowerCase();
      if (type === 'checkbox' || type === 'radio') {
        var isCorr = ctrl.getAttribute('data-correct') === 'true';
        if (fillCorrect) {
          ctrl.checked = isCorr;
        } else if (ctrl.getAttribute('data-auto-filled') === '1') {
          ctrl.checked = false;
        }
        ctrl.setAttribute('data-auto-filled', fillCorrect ? '1' : '0');
      } else {
        var tans = ctrl.getAttribute('data-tans');
        if (tans !== null) {
          ctrl.disabled = fillCorrect;
          if (fillCorrect) {
            ctrl.value = tans;
            ctrl.classList.add('input-filled-correct');
          } else if (ctrl.getAttribute('data-auto-filled') === '1') {
            ctrl.value = '';
            ctrl.classList.remove('input-filled-correct');
          }
          ctrl.setAttribute('data-auto-filled', fillCorrect ? '1' : '0');
        }
      }
    });

    // 3. Typeset MathJax
    if (window.MathJax && window.MathJax.typesetPromise) {
      var root = document.querySelector('.preview-main-panel') || document.body;
      if (window.MathJax.typesetClear) {
        window.MathJax.typesetClear([root]);
      }
      window.MathJax.typesetPromise([root]).catch(function (e) {
        console.warn('MathJax typeset warning:', e);
      });
    }
  }, 25);
};
