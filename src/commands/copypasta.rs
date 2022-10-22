/// I'd just like to interject for a moment...
#[poise::command(slash_command)]
pub async fn linux(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
  let text = concat!(
  "I'd just like to interject for a moment. What you're refering to as Linux, is in fact, GNU/Linux, or as I've recently taken to calling it, ",
  "GNU plus Linux. Linux is not an operating system unto itself, but rather another free component of a fully functioning GNU system made useful by the GNU corelibs, ",
  "shell utilities and vital system components comprising a full OS as defined by POSIX.",
  "\n\n",
  "Many computer users run a modified version of the GNU system every day, without realizing it. Through a peculiar turn of events, the version of GNU which is widely ",
  "used today is often called Linux, and many of its users are not aware that it is basically the GNU system, developed by the GNU Project.",
  "\n\n",
  "There really is a Linux, and these people are using it, but it is just a part of the system they use. Linux is the kernel: the program in the system that allocates ",
  "the machine's resources to the other programs that you run. The kernel is an essential part of an operating system, but useless by itself; it can only function in ",
  "the context of a complete operating system. Linux is normally used in combination with the GNU operating system: the whole system is basically GNU with Linux added, ",
  "or GNU/Linux. All the so-called Linux distributions are really distributions of GNU/Linux!"
  );

  ctx.say(text).await?;
  Ok(())
}

/// I use Alpine!
#[poise::command(slash_command)]
pub async fn linuxresponse(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
  let text = concat!(
  "\"I use Linux as my operating system,\" I state proudly to the unkempt, bearded man. He swivels around in his desk chair with a devilish gleam in his eyes, ready ",
  "to mansplain with extreme precision. \"Actually\", he says with a grin, \"Linux is just the kernel. You use GNU+Linux!\" I don't miss a beat and reply with a smirk, ",
  "\n\n",
  "\"I use Alpine, a distro that doesn't include the GNU Coreutils, or any other GNU code. It's Linux, but it's not GNU+Linux.\"",
  "The smile quickly drops from the man's face. His body begins convulsing and he foams at the mouth and drops to the floor with a sickly thud. As he writhes around he screams ",
  "\"I-IT WAS COMPILED WITH GCC! THAT MEANS IT'S STILL GNU!\" Coolly, I reply \"If windows were compiled with GCC, would that make it GNU?\" I interrupt his response with ",
  "\"-and work is being made on the kernel to make it more compiler-agnostic. Even if you were correct, you won't be for long.\"",
  "\n\n",
  "With a sickly wheeze, the last of the man's life is ejected from his body. He lies on the floor, cold and limp. I've womansplained him to death."
  );
  
  ctx.say(text).await?;
  Ok(())
}
