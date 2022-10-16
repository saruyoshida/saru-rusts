#![no_std]

use core::f32::consts::PI;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
// form cargo advice
use micromath::*;
// Add end

use core::fmt::Write;
use heapless::consts::U16;
use heapless::String;
use wio_referee::WioReferee;

const MARGIN: u32 = 10;

pub struct WioClock {
    clock_face: Circle,
    hours_radians: f32,
    minutes_radians: f32,
    seconds_radians: f32,
    data: String::<U16>,
    referee: WioReferee,
}

impl WioClock {
    pub fn new(display: &impl DrawTarget) -> Self {
        let clock_face = WioClock::create_face(display);
        let data = String::<U16>::new();
        let referee = WioReferee::new();

        let wio_clock = WioClock {
            clock_face,
            hours_radians: 0.0f32,
            minutes_radians: 0.0f32,
            seconds_radians: 0.0f32,
            data,
            referee,
        };
        wio_clock
    }

    fn polar(&self, circle: &Circle, angle: f32, radius_delta: i32) -> Point {
        let radius = circle.diameter as f32 / 2.0 + radius_delta as f32;

        circle.center()
            + Point::new(
                (angle.sin() * radius) as i32,
             -(angle.cos() * radius) as i32,
            )
    }

    /// Converts an hour into an angle in radians.
    fn hour_to_angle(hour: u32) -> f32 {
        // Convert from 24 to 12 hour time.
        let hour = hour % 12;

        (hour as f32 / 12.0) * 2.0 * PI
    }

    /// Converts a sexagesimal (base 60) value into an angle in radians.
    fn sexagesimal_to_angle(value: u32) -> f32 {
        (value as f32 / 60.0) * 2.0 * PI
    }

    fn create_face(target: &impl DrawTarget) -> Circle {
        // The draw target bounding box can be used to determine the size of the display.
        let bounding_box = target.bounding_box();

        let diameter = bounding_box.size.width.min(bounding_box.size.height) - 2 * MARGIN;

        Circle::with_center(bounding_box.center(), diameter)   
    }

    fn draw_face<D>(&self, target: &mut D, clock_face: &Circle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Draw the outer face.
    // Add original    
        let face_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .stroke_color(Rgb565::WHITE)
            .stroke_width(2)
            .build();
    // Add end

        (*clock_face)
        // Mod original
        //        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 2))
            .into_styled(face_style)
        // Mod end
            .draw(target)?;

        // Draw 12 graduations.
        for angle in (0..12).map(WioClock::hour_to_angle) {
            // Start point on circumference.
            let start = self.polar(clock_face, angle, 0);

            // End point offset by 10 pixels from the edge.
            let end = self.polar(clock_face, angle, -10);

            Line::new(start, end)
                .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
                .draw(target)?;
        }

        Ok(())
    }

    fn draw_hand<D>(
        &self, 
        target: &mut D,
        clock_face: &Circle,
        angle: f32,
        length_delta: i32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let end = self.polar(clock_face, angle, length_delta);

        Line::new(clock_face.center(), end)
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
            .draw(target)
    }


       
    fn draw_second_decoration<D>(
        &self, 
        target: &mut D,
        clock_face: &Circle,
        angle: f32,
        length_delta: i32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let decoration_position = self.polar(clock_face, angle, length_delta);

        let decoration_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .stroke_color(Rgb565::WHITE)
            .stroke_width(1)
            .build();

        // Draw a fancy circle near the end of the second hand.
        Circle::with_center(decoration_position, 11)
            .into_styled(decoration_style)
            .draw(target)
    }

    fn draw_digital_clock<D>(
        &self, 
        target: &mut D,
        clock_face: &Circle,
        time_str: &str,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Create a styled text object for the time text.
        let mut text = Text::new(
            &time_str,
         Point::zero(),
            MonoTextStyle::new(&FONT_9X15, Rgb565::BLACK),
        );

        // Move text to be centered between the 12 o'clock point and the center of the clock face.
        text.translate_mut(
            clock_face.center()
            - text.bounding_box().center()
                - clock_face.bounding_box().size.y_axis() / 4,
        );

        // Add a background around the time digits.
        // Note that there is no bottom-right padding as this is added by the font renderer itself.
        let text_dimensions = text.bounding_box();
        Rectangle::new(
            text_dimensions.top_left - Point::new(3, 3),
            text_dimensions.size + Size::new(4, 4),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(target)?;

        // Draw the text after the background is drawn.
        text.draw(target)?;

        Ok(())
    }

    fn draw_inner_circle<D>(
        &self, 
        target: &mut D,
        clock_face: &Circle,
        length_delta: i32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let radius = clock_face.diameter as f32 + length_delta as f32;
 Circle::with_center(clock_face.center(), radius as u32)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(target)
    }

    // Add end
    pub fn update(
        &mut self,
        hours: u8,
        minutes: u8,
        seconds: u8,
    ) {
       self.referee.judgment((hours as i32,
                             minutes as i32,
                             seconds as i32));

       if self.referee.as_update() {
           let mut data =String::<U16>::new(); 
           write!(
                data,
                "{:02}:{:02}:{:02}",
                hours, minutes, seconds
           )
           .ok()
           .unwrap();

           self.data = data;

           self.hours_radians = WioClock::hour_to_angle(hours as u32);
           self.minutes_radians = WioClock::sexagesimal_to_angle(minutes as u32);
           self.seconds_radians = WioClock::sexagesimal_to_angle(seconds as u32);
       }
    }
}
 
impl Drawable for WioClock
{
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, display: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // 各ヘルパーメソッドを呼び出して、ケース、ボタン、画像を描画する
        if self.referee.as_update() {
            if self.referee.as_first() {
                self.draw_face(display, &self.clock_face)?;
            } else {
                            self.draw_inner_circle(display, &self.clock_face, -11)?;
            }

            self.draw_hand(display, &self.clock_face, self.hours_radians, -60)?;
            self.draw_hand(display, &self.clock_face, self.minutes_radians, -30)?;
            self.draw_hand(display, &self.clock_face, self.seconds_radians, -11)?;
          self.draw_second_decoration(display, &self.clock_face, self.seconds_radians, -30)?;
    
        // Draw digital clock just above center.
            self.draw_digital_clock(display, &self.clock_face, &self.data)?;
    
        // Draw a small circle over the hands in the center of the clock face.
        // This has to happen after the hands are drawn so they're covered up.
            Circle::with_center(self.clock_face.center(), 9)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(display)
        } else {
            Ok(())
        }
    }
}            
    