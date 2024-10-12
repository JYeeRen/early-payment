use chrono::{Datelike, NaiveDate};
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
struct Loan {
    principal: Decimal,
    annual_rate: Decimal,
    done_months: u32,
    months: u32,
    start_date: NaiveDate,
    monthly_principal_payment: Decimal,
}

#[derive(Debug, Clone)]
struct PaymentSchedule {
    period: u32,
    interest: Decimal,
    principal_payment: Decimal,
    remaining_principal: Decimal,
    total_payment: Decimal,
    interest_rate: Decimal,
    payment_date: NaiveDate,
    early_payment: Option<Decimal>,
}

impl Loan {
    fn new(
        principal: Decimal,
        annual_rate: Decimal,
        done_months: u32,
        months: u32,
        start_date: NaiveDate,
    ) -> Self {
        let monthly_principal_payment = (principal / Decimal::from(months - done_months)).round_dp(2);
        Self {
            principal,
            annual_rate,
            done_months,
            months,
            start_date,
            monthly_principal_payment,
        }
    }

    fn generate_schedule(&self) -> Vec<PaymentSchedule> {
        let mut schedule = Vec::new();
        let months = self.months - self.done_months;
        let monthly_principal_payment = self.monthly_principal_payment;
        let mut remaining_principal = self.principal;
        let monthly_rate = self.annual_rate / Decimal::from(12) / Decimal::from(100);
        let mut current_date = self.start_date;

        for period in 1..=months {
            let interest = (remaining_principal * monthly_rate).round_dp(2);
            let total_payment = (monthly_principal_payment + interest).round_dp(2);

            let monthly_principal_payment = if remaining_principal < monthly_principal_payment {
                remaining_principal
            } else {
                monthly_principal_payment
            };

            schedule.push(PaymentSchedule {
                period: period + self.done_months,
                interest,
                principal_payment: monthly_principal_payment,
                remaining_principal: remaining_principal,
                total_payment,
                interest_rate: self.annual_rate,
                payment_date: current_date,
                early_payment: None,
            });

            remaining_principal -= monthly_principal_payment;

            current_date = current_date
                .with_month((current_date.month0() + 1) % 12 + 1)
                .and_then(|date| {
                    date.with_year(current_date.year() + (current_date.month0() + 1) as i32 / 12)
                })
                .expect("Failed to calculate date");
        }

        schedule
    }

    fn adjust_rate(
        &mut self,
        new_rate: Decimal,
        from_period: u32,
        schedule: &mut Vec<PaymentSchedule>,
    ) {
        self.annual_rate = new_rate;

        for payment in schedule.iter_mut().skip((from_period - 1) as usize) {
            payment.interest_rate = new_rate;
            let monthly_rate = new_rate / Decimal::from(12) / Decimal::from(100);
            payment.interest = (payment.remaining_principal * monthly_rate).round_dp(2);
            payment.total_payment = payment.principal_payment + payment.interest;
        }
    }

    fn make_early_payment(
        &mut self,
        extra_payment: Decimal,
        period: u32,
        shorten_term: bool,
        schedule: &mut Vec<PaymentSchedule>,
    ) {
        let mut idx: u32 = period - self.done_months - 1;
        if idx < 0 as u32 {
            return;
        }

        if idx as usize >= schedule.len() {
            return;
        }

        let mut remaining_principal = (schedule[idx as usize].remaining_principal - extra_payment).round_dp(2);

        if remaining_principal < Decimal::from(0) {
            return;
        }

        schedule[idx as usize].early_payment = Some(extra_payment);

        if shorten_term {
            for payment in &mut schedule[idx as usize..] {
                let monthly_rate = payment.interest_rate / Decimal::from(12) / Decimal::from(100);
                let interest = (remaining_principal * monthly_rate).round_dp(2);

                payment.remaining_principal = remaining_principal;
                payment.principal_payment = if remaining_principal < payment.principal_payment {
                    remaining_principal
                } else {
                    payment.principal_payment
                };
                payment.interest = interest;
                payment.total_payment = (payment.principal_payment + interest).round_dp(2);
    
                remaining_principal -= payment.principal_payment;

                idx += 1;

                if remaining_principal.is_zero() {
                    schedule.truncate(idx as usize);
                    break;
                }
            }
        }

        if !shorten_term {
            let remaining_period = self.months - schedule[idx as usize].period + 1;
            
            self.monthly_principal_payment = (remaining_principal / Decimal::from(remaining_period)).round_dp(2);

            for payment in &mut schedule[idx as usize..] {
                let monthly_rate = payment.interest_rate / Decimal::from(12) / Decimal::from(100);
                let interest = (remaining_principal * monthly_rate).round_dp(2);

                payment.remaining_principal = remaining_principal;
                payment.principal_payment = self.monthly_principal_payment;
                payment.principal_payment = if remaining_principal < payment.principal_payment {
                    remaining_principal
                } else {
                    payment.principal_payment
                };
                payment.interest = interest;
                payment.total_payment = (payment.principal_payment + interest).round_dp(2);
    
                remaining_principal -= payment.principal_payment;

                idx += 1;
            }
        }
    }

    fn total_interest_paid(&self, schedule: &Vec<PaymentSchedule>) -> Decimal {
        schedule.iter().map(|p| p.interest).sum()
    }

    // fn find_remaining_schedule<'a>(
    //     &self,
    //     schedule: &'a mut Vec<PaymentSchedule>,
    //     period: u32,
    // ) -> &'a mut [PaymentSchedule] {
    //     &mut schedule[period as usize - 1..]
    // }
}

fn main() {
    let start_date = NaiveDate::from_ymd_opt(2024, 10, 19).expect("Invalid date provided");

    let loan = Loan::new(
        Decimal::from_str("536714.20").unwrap(),
        Decimal::from_str("4.2").unwrap(),
        57,
        288,
        start_date,
    );

    let loan2 = Loan::new(
        Decimal::from_str("536714.20").unwrap(),
        Decimal::from_str("4.2").unwrap(),
        57,
        288,
        start_date,
    );

    let mut schedule = loan.generate_schedule();

    let mut schedule2 = loan2.generate_schedule();

    // Example: Adjust rate at a certain period
    let mut loan_clone = loan.clone();
    let mut loan_clone2 = loan2.clone();

    loan_clone.adjust_rate(Decimal::from_str("3.9").unwrap(), 2, &mut schedule);
    loan_clone.adjust_rate(Decimal::from_str("3.55").unwrap(), 3, &mut schedule);
    
    loan_clone2.adjust_rate(Decimal::from_str("3.9").unwrap(), 2, &mut schedule2);
    loan_clone2.adjust_rate(Decimal::from_str("3.55").unwrap(), 3, &mut schedule2);

    loan_clone.make_early_payment(
        (loan_clone.monthly_principal_payment * Decimal::from(43)).round_dp(2),
        58,
        true,
        &mut schedule
    );
    
    loan_clone2.make_early_payment(
        (loan_clone2.monthly_principal_payment * Decimal::from(43)).round_dp(2),
        58,
        true,
        &mut schedule2
    );

    for period in 0..=schedule.len() {
        if (59 + period) % 3 == 0 {
            let payment = (Decimal::from(10000) / loan_clone.monthly_principal_payment).trunc() * loan_clone.monthly_principal_payment;
            let payment2 = (Decimal::from(10000) / loan_clone2.monthly_principal_payment).trunc() * loan_clone2.monthly_principal_payment;

            loan_clone.make_early_payment(
                payment,
                59 + period as u32,
                true,
                &mut schedule
            );

            loan_clone2.make_early_payment(
                payment2,
                59 + period as u32,
                false,
                &mut schedule2
            );
        }

        if period > 12 && period % 12 == 0 {
            let payment2 = (Decimal::from(10000) / loan_clone2.monthly_principal_payment).trunc() * loan_clone2.monthly_principal_payment;
            loan_clone2.make_early_payment(
                payment2,
                59 + period as u32,
                false,
                &mut schedule2
            );
        }
    }

    println!("缩短期限 {}", loan_clone.total_interest_paid(&schedule));
    println!("减少月供 {}", loan_clone2.total_interest_paid(&schedule2));

    println!("Period\tRemaining Balance\tMonth\tRate\tInterest\tPrincipal\tPayment\t\tEarly Payment");
    println!("-----------------------------------------------------------");
    for p in &schedule2 {
        println!(
            "{}\t{:<8}\t{}\t{}\t{:<8}\t{:<8}\t{:<8}\t{:<8}",
            p.period,
            p.remaining_principal,
            p.payment_date,
            p.interest_rate,
            p.interest,
            p.principal_payment,
            p.total_payment,
            p.early_payment
                .map_or_else(|| "None".to_string(), |v| v.to_string()),
        );
    }
}
