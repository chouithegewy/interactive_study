use std::cmp::{max, min};
use std::io::{self, Read, Write};
use std::process::Command;

#[derive(Clone, Copy)]
struct StudyCard {
    section: &'static str,
    prompt: &'static str,
    answer: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Cards,
    Quiz,
    Outline,
}

struct App {
    cards: Vec<StudyCard>,
    current: usize,
    mode: Mode,
    show_answer: bool,
    done: Vec<bool>,
    filter: Option<&'static str>,
    message: &'static str,
}

struct RawTerminal;

impl RawTerminal {
    fn enter() -> io::Result<Self> {
        let _ = Command::new("stty")
            .args(["-icanon", "-echo", "min", "1", "time", "0"])
            .status();
        print!("\x1b[?1049h\x1b[?25l");
        io::stdout().flush()?;
        Ok(Self)
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = Command::new("stty").args(["sane"]).status();
        print!("\x1b[?25h\x1b[?1049l");
        let _ = io::stdout().flush();
    }
}

impl App {
    fn new() -> Self {
        let cards = study_cards();
        let done = vec![false; cards.len()];
        Self {
            cards,
            current: 0,
            mode: Mode::Cards,
            show_answer: false,
            done,
            filter: None,
            message: "Use arrows/j/k to move. Space flips answers. q quits.",
        }
    }

    fn active_indices(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, card)| self.filter.map_or(true, |section| card.section == section))
            .map(|(index, _)| index)
            .collect()
    }

    fn current_index(&self) -> usize {
        let active = self.active_indices();
        if active.is_empty() {
            0
        } else {
            active[self.current % active.len()]
        }
    }

    fn current_card(&self) -> StudyCard {
        self.cards[self.current_index()]
    }

    fn next(&mut self) {
        let len = self.active_indices().len();
        if len > 0 {
            self.current = (self.current + 1) % len;
            self.show_answer = false;
        }
    }

    fn previous(&mut self) {
        let len = self.active_indices().len();
        if len > 0 {
            self.current = if self.current == 0 {
                len - 1
            } else {
                self.current - 1
            };
            self.show_answer = false;
        }
    }

    fn toggle_done(&mut self) {
        let index = self.current_index();
        self.done[index] = !self.done[index];
        self.message = if self.done[index] {
            "Marked as reviewed."
        } else {
            "Marked as not reviewed."
        };
    }

    fn set_filter_by_number(&mut self, number: u8) {
        const SECTIONS: [&str; 8] = [
            "Network Layer",
            "Transport Utilities",
            "UDP",
            "Reliable Transfer",
            "TCP",
            "HTTP/FTP",
            "Email",
            "HW3 TCP Practice",
        ];

        if number == 0 {
            self.filter = None;
            self.current = 0;
            self.message = "Showing all sections.";
            return;
        }

        let index = usize::from(number.saturating_sub(1));
        if let Some(section) = SECTIONS.get(index) {
            self.filter = Some(section);
            self.current = 0;
            self.show_answer = false;
            self.message = "Section filter changed.";
        }
    }
}

fn main() -> io::Result<()> {
    let _terminal = RawTerminal::enter()?;
    let mut app = App::new();
    let mut stdin = io::stdin();

    loop {
        draw(&app)?;
        let mut byte = [0_u8; 1];
        stdin.read_exact(&mut byte)?;

        match byte[0] {
            b'q' | 3 => break,
            b'j' | b'n' | b'\r' => app.next(),
            b'k' | b'p' => app.previous(),
            b' ' => app.show_answer = !app.show_answer,
            b'm' => app.toggle_done(),
            b'c' => {
                app.mode = Mode::Cards;
                app.show_answer = false;
                app.message = "Card mode.";
            }
            b'z' => {
                app.mode = Mode::Quiz;
                app.show_answer = false;
                app.message = "Quiz mode. Answer first, then press Space.";
            }
            b'o' => {
                app.mode = Mode::Outline;
                app.message = "Outline mode.";
            }
            b'0'..=b'8' => app.set_filter_by_number(byte[0] - b'0'),
            27 => handle_escape(&mut stdin, &mut app)?,
            _ => {}
        }
    }

    Ok(())
}

fn handle_escape(stdin: &mut io::Stdin, app: &mut App) -> io::Result<()> {
    let mut seq = [0_u8; 2];
    if stdin.read_exact(&mut seq).is_ok() && seq[0] == b'[' {
        match seq[1] {
            b'C' | b'B' => app.next(),
            b'D' | b'A' => app.previous(),
            _ => {}
        }
    }
    Ok(())
}

fn draw(app: &App) -> io::Result<()> {
    let (rows, cols) = terminal_size();
    let width = min(cols, 80).max(20);
    print!("\x1b[2J\x1b[H");

    draw_header(app, width);
    match app.mode {
        Mode::Cards => draw_card_mode(app, width),
        Mode::Quiz => draw_quiz_mode(app, width),
        Mode::Outline => draw_outline_mode(app, width, rows),
    }
    draw_footer(app, width);

    io::stdout().flush()
}

fn draw_header(app: &App, width: usize) {
    let reviewed = app.done.iter().filter(|done| **done).count();
    let filter = app.filter.unwrap_or("All sections");
    let mode = match app.mode {
        Mode::Cards => "Cards",
        Mode::Quiz => "Quiz",
        Mode::Outline => "Outline",
    };

    println!("CS4470 Final Review");
    println!(
        "{}",
        fit_line(
            &format!(
                "{mode} | {filter} | {reviewed}/{} reviewed",
                app.cards.len()
            ),
            width
        )
    );
    println!();
}

fn draw_card_mode(app: &App, width: usize) {
    let active = app.active_indices();
    let card = app.current_card();
    let status = if app.done[app.current_index()] {
        "[reviewed]"
    } else {
        "[open]"
    };

    println!();
    println!(
        "{}",
        fit_line(
            &format!(
                "Card {}/{} {} | {}",
                app.current + 1,
                max(active.len(), 1),
                status,
                card.section
            ),
            width
        )
    );
    print_wrapped("Study prompt:", card.prompt, width);
    println!();
    print_wrapped("What to know:", card.answer, width);
}

fn draw_quiz_mode(app: &App, width: usize) {
    let active = app.active_indices();
    let card = app.current_card();
    println!();
    println!(
        "{}",
        fit_line(
            &format!(
                "Question {}/{} | {}",
                app.current + 1,
                max(active.len(), 1),
                card.section
            ),
            width
        )
    );
    print_wrapped("Question:", card.prompt, width);
    println!();
    if app.show_answer {
        print_wrapped("Answer:", card.answer, width);
    } else {
        println!("Press Space when you are ready to reveal the answer.");
    }
}

fn draw_outline_mode(app: &App, width: usize, rows: usize) {
    println!();
    println!("Filters");
    println!(
        "{}",
        fit_line("0 All  1 Network  2 Utilities  3 UDP  4 Reliable", width)
    );
    println!(
        "{}",
        fit_line("5 TCP  6 HTTP/FTP  7 Email  8 HW3 Practice", width)
    );
    println!();
    println!("Review outline");

    let active = app.active_indices();
    let max_items = rows.saturating_sub(13).max(5);
    let selected = app.current.min(active.len().saturating_sub(1));
    let start = selected.saturating_sub(max_items / 2);
    let end = min(start + max_items, active.len());

    if start > 0 {
        println!("  ... {} earlier cards", start);
    }

    let mut last_section = "";
    for (offset, index) in active[start..end].iter().copied().enumerate() {
        let card = app.cards[index];
        if card.section != last_section {
            last_section = card.section;
            println!();
            println!("{last_section}");
        }
        let marker = if app.done[index] { "x" } else { " " };
        let cursor = if start + offset == selected { ">" } else { " " };
        println!(
            "{cursor} [{marker}] {}",
            fit_line(card.prompt, width.saturating_sub(6))
        );
    }

    if end < active.len() {
        println!("  ... {} later cards", active.len() - end);
    }
}

fn draw_footer(app: &App, width: usize) {
    println!();
    println!(
        "{}",
        fit_line("c cards  z quiz  o outline  0-8 filter", width)
    );
    println!(
        "{}",
        fit_line("arrows/j/k move  space flip  m mark  q quit", width)
    );
    println!("{}", fit_line(app.message, width));
}

fn print_wrapped(label: &str, text: &str, width: usize) {
    println!("{label}");
    for line in wrap_text(text, width.saturating_sub(2)) {
        println!("  {line}");
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = max(width, 20);
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.is_empty() {
                line.push_str(word);
            } else if line.len() + word.len() + 1 <= width {
                line.push(' ');
                line.push_str(word);
            } else {
                lines.push(line);
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }

    lines
}

fn fit_line(text: &str, width: usize) -> String {
    if text.len() <= width {
        text.to_string()
    } else if width <= 3 {
        ".".repeat(width)
    } else {
        let cutoff = width - 3;
        format!("{}...", &text[..cutoff])
    }
}

fn terminal_size() -> (usize, usize) {
    let output = Command::new("stty").arg("size").output();
    if let Ok(output) = output {
        if let Ok(text) = String::from_utf8(output.stdout) {
            let parts: Vec<_> = text.split_whitespace().collect();
            if parts.len() == 2 {
                let rows = parts[0].parse().unwrap_or(28);
                let cols = parts[1].parse().unwrap_or(88);
                return (rows, cols);
            }
        }
    }
    (28, 88)
}

fn study_cards() -> Vec<StudyCard> {
    vec![
        StudyCard {
            section: "Network Layer",
            prompt: "Compare link-state and distance-vector routing algorithms.",
            answer: "Link-state routers flood topology information and compute shortest paths locally. Distance-vector routers exchange path-cost vectors with neighbors and update routes from neighbor advertisements.",
        },
        StudyCard {
            section: "Network Layer",
            prompt: "Know the Internet routing protocol split: intra-AS vs. inter-AS.",
            answer: "Intra-AS routing runs inside one autonomous system, such as RIP or OSPF. Inter-AS routing runs between autonomous systems, primarily BGP.",
        },
        StudyCard {
            section: "Network Layer",
            prompt: "What are RIP and OSPF used for?",
            answer: "Both are intra-AS routing protocols. RIP is distance-vector based, while OSPF is link-state based and computes routes from advertised link-state information.",
        },
        StudyCard {
            section: "Network Layer",
            prompt: "What is BGP used for?",
            answer: "BGP is the inter-AS routing protocol used to exchange reachability information and policy-driven routes between autonomous systems.",
        },
        StudyCard {
            section: "Transport Utilities",
            prompt: "What is ICMP for, and how is an ICMP message transmitted?",
            answer: "ICMP reports network-layer errors and diagnostic information. ICMP messages are carried inside IP datagrams rather than using TCP or UDP.",
        },
        StudyCard {
            section: "Transport Utilities",
            prompt: "How does ICMP avoid infinite error-message loops?",
            answer: "Routers and hosts do not send ICMP error messages in response to ICMP error messages, and they avoid generating errors for cases such as certain broadcast or fragmented packets.",
        },
        StudyCard {
            section: "Transport Utilities",
            prompt: "Why is NAT used, and how does it work?",
            answer: "NAT lets private hosts share one or a few public IP addresses. The NAT device rewrites source addresses/ports on outbound packets and uses a translation table to map replies back to internal hosts.",
        },
        StudyCard {
            section: "Transport Utilities",
            prompt: "What is DHCP for, and what is the basic exchange?",
            answer: "DHCP automatically gives hosts network configuration such as IP address, subnet mask, gateway, and DNS server. The common sequence is Discover, Offer, Request, Acknowledge.",
        },
        StudyCard {
            section: "UDP",
            prompt: "What services does UDP provide?",
            answer: "UDP provides connectionless process-to-process delivery with ports, length, and checksum support. It does not provide reliability, ordering, congestion control, or flow control.",
        },
        StudyCard {
            section: "UDP",
            prompt: "Which kinds of applications commonly use UDP?",
            answer: "Applications that value low overhead or can tolerate loss often use UDP, including DNS, streaming media, VoIP, online games, and some request/response protocols.",
        },
        StudyCard {
            section: "UDP",
            prompt: "What UDP header fields matter most?",
            answer: "Source port, destination port, length, and checksum. The checksum helps detect corruption across the UDP segment and selected IP pseudo-header fields.",
        },
        StudyCard {
            section: "UDP",
            prompt: "How do you compute the UDP checksum conceptually?",
            answer: "Add 16-bit words using one's-complement arithmetic, wrap carries around, then take the one's complement of the final sum. The receiver repeats the check to detect corruption.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "What errors must reliable transfer handle?",
            answer: "Reliable transfer must handle corrupted segments and lost segments, usually with checksums, acknowledgments, sequence numbers, and retransmissions.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "What is ARQ?",
            answer: "Automatic Repeat reQuest is a reliability strategy where the receiver or sender logic causes retransmission when loss or corruption is detected, commonly using ACKs, NAKs, timers, and sequence numbers.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "What are the pros and cons of stop-and-wait?",
            answer: "It is simple and can provide complete reliable transfer, but it is inefficient because the sender waits idle for an ACK before sending the next segment.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "Why does sending multiple segments require more than one sequence-number bit?",
            answer: "With multiple outstanding segments, the receiver and sender must distinguish several in-flight packets and retransmissions, so a single alternating bit is not enough.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "How does Go-Back-N use a sliding window?",
            answer: "The sender may transmit up to N unacknowledged segments. The number it can still send is N minus the distance between nextseqnum and send_base.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "What do cumulative ACKs mean in Go-Back-N?",
            answer: "ACK n acknowledges all segments up to and including sequence number n - 1, so the sender can slide the base forward past all cumulatively acknowledged data.",
        },
        StudyCard {
            section: "Reliable Transfer",
            prompt: "How are timers used in Go-Back-N?",
            answer: "GBN uses one timer for the oldest transmitted but unacknowledged segment. On timeout, the sender retransmits that segment and all later outstanding segments.",
        },
        StudyCard {
            section: "TCP",
            prompt: "What services does TCP provide?",
            answer: "TCP provides reliable, ordered byte-stream delivery, connection management, flow control, congestion control, full-duplex data transfer, and demultiplexing through ports.",
        },
        StudyCard {
            section: "TCP",
            prompt: "Which TCP header fields are highlighted in the review?",
            answer: "SYN and FIN flags for connection management, sequence and acknowledgment numbers for reliable byte-stream tracking, and receive window for flow control.",
        },
        StudyCard {
            section: "TCP",
            prompt: "How does TCP detect data loss?",
            answer: "TCP detects likely loss through timeout expiration and duplicate ACKs. Duplicate ACKs suggest later data arrived while an earlier segment is missing.",
        },
        StudyCard {
            section: "TCP",
            prompt: "How is TCP timeout estimated?",
            answer: "TCP estimates RTT, smooths it over time, tracks RTT variation, and sets timeout to a value above the estimated RTT to avoid premature retransmission.",
        },
        StudyCard {
            section: "TCP",
            prompt: "How does TCP flow control work?",
            answer: "The receiver advertises available buffer space in the receive window. The sender limits unacknowledged data so it does not overflow the receiver.",
        },
        StudyCard {
            section: "TCP",
            prompt: "Flow control vs. congestion control: what is the difference?",
            answer: "Flow control protects the receiver's buffer. Congestion control protects the network by adapting the sending rate to perceived congestion.",
        },
        StudyCard {
            section: "TCP",
            prompt: "What happens in TCP's three-way handshake?",
            answer: "Client sends SYN, server replies SYN-ACK, client sends ACK. Both sides establish initial sequence numbers and agree the connection is open.",
        },
        StudyCard {
            section: "TCP",
            prompt: "What is TCP connection teardown about?",
            answer: "TCP teardown closes each direction of the full-duplex byte stream, commonly using FIN and ACK exchanges so both sides can finish sending outstanding data.",
        },
        StudyCard {
            section: "TCP",
            prompt: "When do slow start and congestion avoidance run?",
            answer: "Slow start begins at connection start or after timeout and grows the congestion window quickly until a threshold. Congestion avoidance then grows more conservatively.",
        },
        StudyCard {
            section: "TCP",
            prompt: "How does TCP adjust after timeout vs. three duplicate ACKs?",
            answer: "On timeout, TCP treats congestion as severe, reduces the threshold, and restarts with a small congestion window. With three duplicate ACKs, TCP performs fast retransmit and reduces the window less drastically.",
        },
        StudyCard {
            section: "HTTP/FTP",
            prompt: "What transport service does HTTP use, and what does stateless mean?",
            answer: "HTTP uses TCP. Stateless means each request is handled independently; the protocol does not inherently remember prior client state between requests.",
        },
        StudyCard {
            section: "HTTP/FTP",
            prompt: "Non-persistent vs. persistent HTTP connections: what changes?",
            answer: "Non-persistent HTTP opens a new TCP connection for each object. Persistent HTTP reuses a TCP connection for multiple request/response exchanges.",
        },
        StudyCard {
            section: "HTTP/FTP",
            prompt: "How do HTTP cookies work?",
            answer: "A server sends a cookie value to the browser, the browser stores it, and later requests include it so the server can associate requests with user/session state.",
        },
        StudyCard {
            section: "HTTP/FTP",
            prompt: "What is FTP for, and why does it use two connections?",
            answer: "FTP transfers files. It uses a control connection for commands/replies and a data connection for file or directory data transfer.",
        },
        StudyCard {
            section: "Email",
            prompt: "What are the main email system components?",
            answer: "User agents, mail servers, and SMTP. User agents compose/read mail, mail servers store and relay it, and SMTP pushes mail between sending components.",
        },
        StudyCard {
            section: "Email",
            prompt: "Which protocol is used to send email?",
            answer: "SMTP is used from the sender's user agent to the sender's mail server and from the sender's mail server to the receiver's mail server.",
        },
        StudyCard {
            section: "Email",
            prompt: "Which protocols are used to receive email?",
            answer: "POP3 or IMAP is used between the receiver's mail server and receiver's user agent.",
        },
        StudyCard {
            section: "Email",
            prompt: "What is the difference between POP3 and IMAP?",
            answer: "POP3 commonly downloads messages for local management. IMAP keeps mail organized on the server and synchronizes folders and message state across clients.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "A TCP receive window holds bytes 1001 through 4000, and the next byte to send is 2001. After ACK 1500 with advertised window 3000, what range is in the sender window?",
            answer: "ACK 1500 means bytes through 1499 are acknowledged, so the window base moves to 1500. With advertised window 3000, the sender window spans bytes 1500 through 4499.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "After that ACK/window update, a 1000-byte segment is sent. How many bytes can still be sent without waiting for new ACKs?",
            answer: "The next byte after sending 1000 bytes from 2001 is 3001. The window ends at 4499, so available bytes are 4499 - 3001 + 1 = 1499.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "If A and B have initial sequence numbers 3000 and 5000, what sequence and ACK numbers does A use for a first 200-byte payload?",
            answer: "A uses Seq = 3001 and Ack = 5001. The first data byte follows A's initial sequence number, and A acknowledges B's initial sequence number plus one.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "After A sends 200 bytes starting at Seq 3001, B sends a 100-byte payload. What sequence and ACK numbers does B use?",
            answer: "B uses Seq = 5001 and Ack = 3201. A's 200 bytes cover sequence numbers 3001 through 3200, so the next expected byte from A is 3201.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "A then sends a lost 100-byte payload after receiving B's ACK. What sequence and ACK numbers are on that lost segment?",
            answer: "A uses Seq = 3201 and Ack = 5101. B's 100-byte payload covers 5001 through 5100, so A acknowledges the next expected byte, 5101.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "Right after the lost 100-byte segment, A sends another 250-byte payload. What sequence and ACK numbers are used?",
            answer: "A uses Seq = 3301 and Ack = 5101. Sequence 3301 follows the 100 bytes that began at 3201, even though that previous segment was lost.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "B receives A's later segment but is still missing the earlier bytes. What ACK does B send with its next 100-byte payload?",
            answer: "B uses Seq = 5101 and Ack = 3201. The ACK remains 3201 because TCP cumulative ACKs identify the next in-order byte expected.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "In the homework timing problem, what parameters define the stop-and-wait and Go-Back-N transfer diagrams?",
            answer: "There are 12 segments, RTT is 1 ms, each frame takes 0.1 ms to transmit, and timeout is 2 ms. Stop-and-wait has corruption/loss on segments 2 and 6; Go-Back-N uses window size 4 with segment 1 corrupted and segments 3 and 5 lost on first transmission.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "What is the key timing difference between stop-and-wait and Go-Back-N in that 12-segment problem?",
            answer: "Stop-and-wait sends one segment per RTT cycle and stalls on each error. Go-Back-N pipelines up to four outstanding segments, but a timeout forces retransmission from the oldest missing segment onward.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "For a segment carrying bytes 1401 through 1600, what ACK should the receiver send after it arrives in order?",
            answer: "The receiver should ACK 1601, the next byte expected. TCP ACK numbers are cumulative and point to the next missing byte, not the last received byte.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "If bytes 1601-1900 and 1901-2100 are both lost, what should A retransmit after timeout?",
            answer: "A should retransmit from the oldest unacknowledged byte, starting with the segment carrying bytes 1601-1900. Later missing data cannot be cumulatively acknowledged until that gap is filled.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "How are sample RTT, smoothed RTT, RTT deviation, and timeout computed after Segment 1 is sent at 0:00 and ACKed at 0:07?",
            answer: "RTTM = 7 seconds, RTTS = 7, RTTD = 7/2 = 3.5, and RTO = RTTS + 4*RTTD = 7 + 14 = 21 seconds.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "After Segment 3 is sent at 0:20 and ACKed at 0:35, what RTT values does the solution calculate?",
            answer: "RTTM = 15 seconds, RTTS = 7*7/8 + 15*1/8 = 8 seconds, RTTD = 3.5*3/4 + 1/4*(15 - 8) = 4.375, and RTO = 8 + 4*4.375 = 25.5, rounded to about 26 seconds.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "If Segment 5 is sent at 0:37 with RTO about 26 seconds, when should timeout occur and what happens to RTO?",
            answer: "Timeout occurs at 0:63, so Segment 5 should be retransmitted. The timeout value is doubled from about 26 seconds to about 52 seconds.",
        },
        StudyCard {
            section: "HW3 TCP Practice",
            prompt: "Why should the ACK at 0:75 for retransmitted Segment 5 not be used as an RTT measurement?",
            answer: "Because the segment was retransmitted, the sender cannot know whether the ACK corresponds to the original transmission or the retransmission. Karn's rule avoids measuring RTT for retransmitted data.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_line_never_exceeds_requested_width() {
        let text = "abcdefghijklmnopqrstuvwxyz";

        for width in 0..=80 {
            assert!(
                fit_line(text, width).len() <= width,
                "width {width} was exceeded"
            );
        }
    }
}
