# Designing an AI-powered Vietnamese input method for PC

**No production-ready AI-powered Vietnamese input method exists for desktop computers today**, despite 100 million Vietnamese speakers and a rich ecosystem of Vietnamese language models. This represents one of the largest untapped opportunities in input method technology. Current desktop tools (UniKey, EVKey) are purely rule-based — they convert Telex/VNI keystrokes into diacritical marks with zero prediction, zero auto-correction, and zero intelligence. Meanwhile, Chinese input methods like Sogou Pinyin serve 600 million daily users with GPT-powered sentence-level prediction, cloud-enhanced vocabulary, and 94% sentence accuracy. The RWKV-7 architecture, with its constant-memory O(d)-per-token inference, offers an ideal engine for bringing this level of intelligence to Vietnamese input on consumer hardware, running a 0.4B-parameter model in under 15ms per token on a standard CPU.

This report synthesizes research across five domains — Chinese AI IME technology, Vietnamese language input, RWKV architecture, cognitive design principles, and IME technical architecture — to provide a complete blueprint for building this system.

---

## What Chinese AI input methods have already solved

The Chinese input method ecosystem provides the clearest blueprint for AI-powered Vietnamese input because both languages face a fundamental many-to-one mapping problem. In Chinese, **fewer than 500 pinyin syllables map to over 6,000 commonly used characters** — each syllable averaging 12 possible characters. The field has evolved through four technological eras: HMM + Viterbi decoding with trigram language models (2000s), neural network language models merged into n-gram formats (2015), attention-based encoder-decoder models framing pinyin-to-character as machine translation (2018), and GPT/Transformer-based approaches achieving state-of-the-art results (2022–present).

**Sogou Pinyin** dominates with ~70% third-party market share and 600M+ daily active users. Its architecture runs a compact local language model for common conversions while asynchronously querying a cloud engine backed by Tencent's Hunyuan large language model. The cloud model carries a **4GB trigram model plus trigger model** with a 2-million-word vocabulary, achieving self-reported 94% accuracy on short sentences and 84% on long sentences. Sogou's secret weapon isn't just AI — it's web-crawled "cell dictionaries" (over 1 million domain-specific entries), real-time trending word updates from search data, and deep ecosystem integration with WeChat and QQ.

**PinyinGPT** (Tencent AI Lab, ACL 2022) demonstrated a pivotal insight: a frozen GPT model achieves state-of-the-art pinyin-to-character conversion on complete pinyin input without any fine-tuning — the language model's general knowledge of Chinese is sufficient. Google's **Gboard** takes a different approach, using a tiny CIFG-LSTM model with only **1.4 million parameters** and a 10,000-word dictionary, achieving practical next-word prediction in a 1.4MB quantized model running via TensorFlow Lite. This proves that extremely small models can deliver production-quality text prediction.

The critical architectural pattern shared across all successful Chinese IMEs is a **tiered prediction pipeline**: fast n-gram or small neural model for immediate candidates (under 20ms), followed by optional neural re-ranking or cloud enhancement for complex inputs. User personalization, frequency adaptation, and domain-specific vocabularies sit atop this foundation. Microsoft Pinyin's integration of Bing-powered cloud suggestions alongside local predictions exemplifies this hybrid approach.

---

## The Vietnamese input landscape has a massive intelligence gap

### How Vietnamese typing works today

Vietnamese uses three input methods, all purely mechanical keystroke-to-diacritic converters. **Telex**, by far the most popular, exploits unused Latin letters: typing `aa` produces `â`, `ow` produces `ơ`, `dd` produces `đ`, while tone marks use `s` (sắc/rising), `f` (huyền/falling), `r` (hỏi/dipping), `x` (ngã/glottalized rising), and `j` (nặng/low falling). **VNI** uses number keys instead (1–9 for different marks). **VIQR** uses punctuation symbols. All three methods allow deferring diacritics to word-end — typing `duocwj` produces `được` — with the IME placing marks on the correct vowel.

The dominant desktop software, **UniKey** (developed since 1994 by Phạm Kim Long), performs this keystroke-to-diacritic conversion and nothing else. Its core engine is open-source and has been integrated into macOS built-in Vietnamese input, ibus-unikey on Linux, and mobile keyboards. **EVKey** forks UniKey's engine and adds application-specific exclusion rules (solving the notorious problem of `w` triggering `ư` in games). Both have **zero prediction, zero auto-correction, and zero contextual awareness**.

On mobile, **Laban Key** (49 million downloads) offers statistical word suggestions after 1–2 characters, using frequency-based n-gram models rather than neural networks. This is the closest thing to "AI" in Vietnamese input — and it's mobile-only, statistics-only, and word-level only.

### Vietnamese language characteristics that shape IME design

Vietnamese has **6 tones** (ngang, huyền, sắc, hỏi, ngã, nặng), **29 alphabet letters** (standard Latin minus f, j, w, z, plus ă, â, đ, ê, ô, ơ, ư), and approximately **67 distinct letter forms** when tonal diacritics are included. The language produces roughly **6,200 possible orthographic syllables**, each written as a separate space-delimited unit.

The word segmentation challenge is critical for AI input: Vietnamese places spaces between syllables, not words. About **85% of Vietnamese word types are multi-syllable compounds**, yet 80%+ of syllable types also function as standalone words. The sentence "thuế thu nhập cá nhân" (5 syllables) segments as "thuế_thu_nhập cá_nhân" (income tax + individual) — an ambiguity that only context can resolve. State-of-the-art word segmentation achieves ~97% F-measure using CRF/SVM methods, but an IME must perform this in real-time.

The tonal system creates both the primary input overhead and the primary error source. The word `ma` has six distinct meanings depending solely on tone (ghost, but, mother, tomb, code, rice seedling). Vietnamese typing requires approximately **30–50% more keystrokes** than equivalent English text due to diacritics. Many users skip diacritics entirely in casual communication, relying on context — a behavior an AI IME could exploit by accepting toneless input and adding correct diacritics automatically.

### The opportunity: rich models, no IME integration

Vietnam's AI research community has produced a comprehensive model ecosystem that no one has integrated into an input method:

- **PhoGPT-4B** (VinAI, 3.7B parameters, trained on 102B tokens): decoder-only GPT ideal for next-token prediction
- **BARTpho** (seq2seq model): ideal for text generation and error correction
- **PhoBERT** (135M/370M parameters, trained on 20GB text): masked language modeling for word prediction
- A fine-tuned BARTpho model (`bmd1905/vietnamese-correction-v2`) already demonstrates real-time Vietnamese spelling correction, converting `côn viec kin doanh` to `công việc kinh doanh`

The experimental **v7** project (101 GitHub stars) represents the only known attempt at AI Vietnamese input. It uses an abbreviated input scheme where users type only consonants plus tone numbers (`t3t5` → `tưởng tượng`), with a small GPT model resolving ambiguity. While innovative, this approach requires learning an entirely new input method — violating the goal of being easy to learn for existing Vietnamese users.

---

## Why RWKV-7 is the right architecture for this IME

RWKV-7 (codename "Goose") combines Transformer-quality language modeling with RNN-efficient inference — precisely the characteristics an input method needs. Created by Bo Peng and now a Linux Foundation project, RWKV achieves **linear training complexity O(T·d) and constant inference complexity O(d) per token**, compared to Transformers' quadratic O(T²·d) training and growing KV cache during inference.

### The constant-memory advantage is decisive for input methods

During a typing session, users produce tokens continuously over minutes or hours. A Transformer-based model accumulates an ever-growing KV cache — a 3B Transformer uses 6.5GB at 1K context but 32.4GB at 128K context before running out of memory. **RWKV maintains exactly 6.2GB regardless of context length** — whether the user has typed 10 words or 10,000. This constant-memory property means the IME never degrades during long typing sessions, never needs context window management, and never requires reprocessing previous input.

RWKV-7 introduced the **Generalized Delta Rule**, which enables meta-in-context learning via gradient descent at every token. At 3B parameters on the Pile benchmark, RWKV-7 achieves **perplexity 9.6 versus Transformer's 9.8** — actually outperforming attention-based models while using simpler operations. The architecture's inference consists purely of matrix-vector multiplications (no matrix-matrix), making it inherently faster on CPUs where memory bandwidth is the bottleneck.

### The 0.4B sweet spot for Vietnamese input

The recommended model is **RWKV-7-World-0.4B**, a multilingual model trained on 100+ languages including Vietnamese. At Q5_1 quantization, it occupies approximately **300MB of RAM** and delivers an estimated **5–15ms per token on a modern consumer CPU** (Intel i5/Ryzen 5 class). This fits comfortably within the 50ms latency budget for keystroke-level prediction while leaving ample headroom for the rest of the IME pipeline.

| Model size | Quantization | RAM usage | Latency per token | Suitability |
|---|---|---|---|---|
| RWKV-7 0.1B | Q8_0 | ~150 MB | ~2–5 ms | Ultra-fast fallback, basic predictions |
| **RWKV-7 0.4B** | **Q5_1** | **~300 MB** | **~5–15 ms** | **Optimal quality/speed balance** |
| RWKV-7 1.5B | Q8_0 | ~1.5 GB | ~30–60 ms | High quality, marginal latency |
| RWKV-7 2.9B | NF4 | ~2.4 GB | ~80–150 ms | Too slow for per-keystroke use |

The deployment ecosystem is mature. **llama.cpp** now supports RWKV-6/7 models in GGUF format with CPU-optimized AVX2/AVX-512 kernels. **web-rwkv** enables Vulkan-based inference on any GPU including integrated graphics — an AMD Radeon 780M iGPU achieves **23.65 tokens/second** with INT8 quantization of the 2.9B model. **rwkv-mobile** covers Android, iOS, and WebAssembly deployments.

Compared to alternatives, RWKV's advantage is clear. **Mamba** (Selective State Space Model) offers similar O(d) inference but lacks RWKV's deployment infrastructure and multilingual model availability. **Small Transformers** (DistilGPT-2, TinyLlama) perform well on short contexts but accumulate KV cache overhead during extended typing sessions. RWKV's state can also be **saved and restored** — enabling the IME to persist context across application restarts, a feature impossible with Transformer KV caches.

A critical caveat: **no dedicated Vietnamese RWKV model exists**. The World models include Vietnamese but aren't optimized for it. The recommended approach is to **LoRA fine-tune** the 0.4B World model on a large Vietnamese text corpus (10GB+ of conversational, messaging, and formal text). Fine-tuning a 0.4B model requires only ~5.7GB GPU memory and can be accomplished on a single consumer GPU. RWKV also supports **State Tuning** — fine-tuning only the initial hidden state — which produces tiny state files that can be loaded at inference time for Vietnamese optimization.

---

## Design principles that minimize cognitive effort

Research reveals a counterintuitive finding: **word prediction on smartphones shows a negative correlation with typing speed** (large-scale study published in PMC). The cognitive cost of scanning and evaluating prediction lists can outweigh keystroke savings. Quinn and Zhai (CHI 2016) confirmed that mobile keyboard suggestions are "not cognitively free" — they impair average time performance even when reducing keyboard actions. This means a Vietnamese AI IME must not simply bolt on a prediction list. It must fundamentally rethink how predictions are delivered.

### The invisible AI paradigm

The most effective AI input operates invisibly. **Gmail's Smart Compose** exemplifies this: suggestions appear as dimmed gray text inline with the cursor; users accept by pressing Tab or simply ignore them by continuing to type. There is no candidate list, no selection step, no cognitive interruption. The system responds within ~100ms per keystroke and achieves high acceptance rates because the predictions feel like natural extensions of the user's thought.

Apple's autocorrect follows the same principle: corrections happen automatically when the user presses space, with iOS 17+ showing a brief underline so users can tap to revert. The design philosophy is **act first, allow easy reversal** rather than asking for permission.

For the Vietnamese IME, this translates into three concrete design rules:

- **Auto-commit high-confidence predictions** (≥95% confidence) without requiring user confirmation. The prediction simply becomes the typed text when the user presses space.
- **Show inline ghost text** (dimmed gray) for phrase/sentence completions rather than a dropdown candidate list. Users accept with Tab, reject by continuing to type.
- **Limit visible candidates to 3 maximum** when disambiguation is needed. Hick-Hyman Law shows reaction time increases logarithmically with choices; mobile keyboards converged on 3 candidates for good reason.

### Sentence-level prediction saves the most time with the fewest decisions

Character-level prediction creates too many micro-decisions. Word-level prediction saves an average of 3.43 characters per phrase but adds ~2 seconds of cognitive overhead per suggestion evaluation — a net negative for skilled typists. **Sentence and phrase-level prediction** minimizes the number of decision points while maximizing keystroke savings. Chinese whole-sentence pinyin-to-character conversion achieves 90–98% accuracy with modern neural models, reducing an entire sentence to a single accept/reject decision.

For Vietnamese, the equivalent approach is **multi-syllable phrase completion**: after the user types 2–3 syllables, the IME predicts the complete phrase or sentence continuation. For example, typing `thời t` could auto-suggest `thời tiết hôm nay` (today's weather) as inline ghost text. The user presses Tab to accept or continues typing to override. This approach targets a **Keystroke Savings Rate above 45%** (competitive with iPhone's 47% on English) while minimizing the total number of decisions per minute.

Key efficiency metrics to target:

- **KSPC (Keystrokes Per Character)**: below 0.7 (significant prediction contribution vs. 1.0 baseline)
- **KSR (Keystroke Savings Rate)**: above 45%
- **Net WPM improvement**: above 10% after accounting for suggestion evaluation overhead
- **Auto-commit accuracy**: above 95% for committed predictions
- **Suggestion-to-display latency**: below 100ms

---

## Technical architecture for a cross-platform implementation

### The engine-server pattern

The proven architecture, demonstrated by RIME's **librime** (4.2k GitHub stars) and Weasel (6.8k stars), separates thin platform-specific frontends from a shared backend engine server:

```
┌─────────────────────────────────────────────────┐
│              Platform Frontend Layer             │
├────────────┬─────────────┬──────────┬───────────┤
│  Windows   │   macOS     │  Linux   │   Linux   │
│  TSF DLL   │ InputMethod │  IBus    │  Fcitx5   │
│  (C/C++)   │ Kit (Swift) │  Engine  │  Addon    │
└─────┬──────┴──────┬──────┴────┬─────┴─────┬─────┘
      └─────────────┴─────┬─────┴───────────┘
                          │ IPC (named pipes / Unix sockets / D-Bus)
                 ┌────────┴────────┐
                 │  Engine Server  │  (Background process, C++/Rust)
                 ├─────────────────┤
                 │ Telex/VNI Core  │  Deterministic diacritic placement
                 │ RWKV Inference  │  Next-token prediction via llama.cpp
                 │ Phrase Predictor│  Beam search over RWKV logits
                 │ Dictionary/LM   │  n-gram FST + user frequency DB
                 │ User Model      │  Local personalization (encrypted)
                 └─────────────────┘
```

**Windows** uses the Text Services Framework (TSF), a COM-based system where the IME is a DLL registered via `regsvr32`. The Weasel project demonstrates implementing both TSF (`WeaselTSF.dll`) and legacy IMM32 (`WeaselIME.dll`) frontends for maximum compatibility. **macOS** uses InputMethodKit (IMK), a Cocoa framework where the IME runs as a background app bundle installed to `~/Library/Input Methods/`. **Linux** supports both IBus (D-Bus based, Red Hat origin) and Fcitx5 (lower latency, better Wayland support via text-input-v3). Process separation between frontends and the engine server provides crash isolation, simplified updates, and code reuse across platforms.

### Inference pipeline and latency budget

The total latency from keystroke to visible suggestion must stay below **100ms**, with a 50ms target for imperceptible response. The budget breaks down as:

| Component | Budget | Implementation |
|---|---|---|
| Key event capture | 1–5 ms | OS input stack |
| Telex/VNI processing | 1–2 ms | Deterministic rules (Unikey engine) |
| **RWKV inference** | **10–30 ms** | llama.cpp with RWKV-7-0.4B Q5_1 |
| Candidate ranking | 2–5 ms | Re-rank with user frequency + context |
| UI rendering | 5–10 ms | Candidate window / inline ghost text |
| **Total** | **20–50 ms** | Well within perception threshold |

The inference strategy exploits RWKV's RNN-like nature: **maintain the hidden state continuously** as the user types, feeding each new token incrementally. This means each keystroke requires only a single forward pass through the model (one matrix-vector multiplication per layer) rather than reprocessing the entire context. The state persists across the typing session and can be serialized to disk for cross-session context preservation.

A **tiered prediction architecture** provides the fastest possible response:

1. **Tier 1 (under 5ms)**: n-gram frequency lookup from a prebuilt Vietnamese FST dictionary. Produces basic word completions immediately.
2. **Tier 2 (under 30ms)**: RWKV-7 0.4B inference for contextual next-token/phrase prediction. Re-ranks and extends Tier 1 candidates.
3. **Tier 3 (optional, async)**: Cloud API call for rare terms, trending words, or domain-specific vocabulary. Results appear 200–500ms later, updating the candidate list non-disruptively.

### Privacy by design

All AI inference runs **locally by default**. The system detects password fields via TSF/IMKit/IBus context flags and disables AI features entirely for sensitive input. The user dictionary is encrypted at rest using platform-native encryption (DPAPI on Windows, Keychain on macOS). For model improvement, **federated learning** with differential privacy can aggregate learning patterns across users without transmitting raw text — Google's Gboard demonstrated this with a 1.4M-parameter CIFG-LSTM trained across 1.5 million devices over 3,000 rounds, achieving better accuracy than server-trained models. Cloud features, if implemented, must be strictly opt-in with clear data disclosures.

---

## The Vietnamese-specific intelligence layer

Beyond the base RWKV model, the Vietnamese IME needs specialized capabilities that address the language's unique challenges.

### Toneless-to-toned conversion

The most impactful feature would accept **Vietnamese text without any diacritics** and add correct tonal marks using context. Many Vietnamese users already type without diacritics in casual settings — the sentence `nguoi viet nam yeu nuoc` should auto-convert to `người Việt Nam yêu nước`. The BARTpho-based correction model (`bmd1905/vietnamese-correction-v2`) already demonstrates this capability at the sentence level. Integrating a similar but smaller model (distilled to 50–100M parameters) into the prediction pipeline would eliminate the single largest source of typing overhead: the 30–50% extra keystrokes required for diacritics.

This can operate in two modes: a **post-hoc correction mode** where users type freely and diacritics are added on sentence completion, and an **inline prediction mode** where the RWKV model predicts the next fully-diacritized syllable from partially-typed toneless input.

### Multi-syllable compound awareness

Since 85% of Vietnamese word types are multi-syllable compounds, the IME must predict across syllable boundaries. After the user types `thời`, the model should predict `tiết` (weather), `gian` (time), or `đại` (era) based on preceding context, offering these as inline ghost completions. The RWKV model's persistent state naturally captures this cross-syllable context without any special engineering — the hidden state after processing `thời` already encodes the probability distribution over likely continuations.

### Bilingual intelligence

Vietnamese-English code-switching is extremely common, and the top user complaint about existing IMEs is that Telex triggers unwanted diacritics on English words (typing `wow` produces `ươ` transformations). The AI layer should **detect language intent per word** using the RWKV model's logits: if the next-token distribution strongly favors English tokens, temporarily suppress Vietnamese diacritic conversion. This eliminates the need for manual Vietnamese/English switching that plagues UniKey and EVKey users.

### Seamless Telex compatibility

The system must accept standard Telex input that 90%+ of Vietnamese users already know, adding AI as a transparent enhancement layer. Users type exactly as they do today — `duocwj` still produces `được` — but now they also get phrase completions, auto-correction of tone errors, and optional toneless input. The learning curve is zero for existing Vietnamese typists.

---

## Recommended implementation roadmap

**Phase 1 — Vietnamese core with cross-platform frontends.** Build Telex/VNI processing (forking or reimplementing the Unikey engine) with TSF, InputMethodKit, IBus, and Fcitx5 frontends communicating via IPC to a shared engine server. This phase delivers a functional, modern Vietnamese IME equivalent to UniKey/EVKey with better application compatibility. Target: 3 months.

**Phase 2 — RWKV integration for next-syllable prediction.** Embed llama.cpp with a fine-tuned RWKV-7-World-0.4B (Q5_1) model. Implement inline ghost text suggestions with Tab-to-accept. Add n-gram dictionary for Tier 1 fast lookups. Fine-tune the model on 10GB+ of Vietnamese conversational and formal text. Target: 3 months.

**Phase 3 — Toneless input and auto-correction.** Add the toneless-to-toned conversion pipeline using a distilled seq2seq model. Implement real-time spelling and tone correction. Add bilingual detection to suppress false diacritic triggers on English words. Target: 2 months.

**Phase 4 — Personalization and advanced prediction.** Implement local user model learning (encrypted frequency database, recently used phrases). Add sentence-level completion for repetitive patterns. Implement adaptive confidence thresholds for auto-commit based on user typing speed and accuracy. Target: 2 months.

**Phase 5 — Optional cloud features.** Trending vocabulary sync, federated learning for model improvement, and optional cloud-enhanced prediction for rare/domain-specific terms. All opt-in with clear privacy controls. Target: ongoing.

---

## Conclusion

The design space for a Vietnamese AI input method is remarkably well-defined by the convergence of three factors: Chinese IMEs have proven that AI-powered sentence-level prediction works at scale for tonal Asian languages; RWKV-7 provides the ideal inference architecture with constant memory, sub-15ms per-token latency on CPU, and a mature deployment ecosystem; and the Vietnamese NLP community has produced models (PhoGPT, BARTpho, PhoBERT) that demonstrate every capability needed — next-token prediction, error correction, and contextual understanding — but has never integrated them into an IME.

The critical design insight is that **the Vietnamese problem is actually simpler than the Chinese problem**. Chinese IMEs must resolve massive pinyin-to-character ambiguity (500 syllables → 6,000+ characters). Vietnamese diacritics are largely deterministic from Telex keystrokes — the AI layer needs to predict *ahead* (what the user will type next) and correct *errors* (wrong tones, missing diacritics), not perform fundamental disambiguation. This means a smaller model can achieve higher effective accuracy, and the system can default to transparent pass-through when uncertain, making it inherently safer than Chinese sentence-level conversion.

The RWKV-7 0.4B model at Q5_1 quantization, occupying 300MB of RAM and delivering 5–15ms inference per token, represents the optimal starting point — large enough for meaningful Vietnamese prediction, small enough to run invisibly on any PC manufactured in the last decade. Combined with the invisible AI design pattern (inline ghost text, auto-commit on high confidence, zero-friction undo), this architecture can deliver 45%+ keystroke savings while requiring zero learning curve from the 100 million Vietnamese typists already fluent in Telex.